//! Session-scoped, acknowledged hot configuration. Disk I/O and parsing never
//! run on the render thread; a bounded mailbox commits at a tick boundary.
use std::sync::{mpsc, Mutex};
use bevy::prelude::*;
use rc_game::weapon_evo::Evolution;

#[derive(Resource)]
pub(crate) struct LiveConfig {
    incoming: Mutex<mpsc::Receiver<(u64, Result<Evolution,String>)>>,
    replies: mpsc::SyncSender<(u64,Result<(),String>)>,
    pub(crate) latest: Option<Evolution>,
}
pub struct LiveConfigPlugin;
impl Plugin for LiveConfigPlugin {
    fn build(&self,app:&mut App) {
        let Ok(root)=std::env::var("RC_WEAPON_LIVE_DIR") else{return;};
        if cfg!(feature="weapon-evo-release") {return;}
        let (tx,rx)=mpsc::sync_channel(1);let (ack_tx,ack_rx)=mpsc::sync_channel(1);
        app.insert_resource(LiveConfig{incoming:Mutex::new(rx),replies:ack_tx,latest:None})
            .add_systems(PreUpdate, apply);
        std::thread::spawn(move|| {
            let root=std::path::PathBuf::from(root);let mut seen=0u64;
            let mut pending_ack=None;
            loop {
                if pending_ack.is_none(){pending_ack=ack_rx.try_recv().ok();}
                if let Some((id,result))=pending_ack.as_ref() {
                    let message=match result {Ok(())=>format!("{id}\nok\n"),Err(e)=>format!("{id}\nerror\n{e}\n")};
                    let tmp=root.join(format!("live-{id}.ack.tmp"));
                    let destination=root.join(format!("live-{id}.ack"));
                    // Every request and acknowledgement has its own immutable
                    // path. Windows readers never race a replacement/deletion.
                    // A transient publish failure retries without reapplying.
                    if destination.is_file() || (std::fs::write(&tmp,message).is_ok() && std::fs::rename(&tmp,&destination).is_ok()){pending_ack=None;}
                }
                let id=std::fs::read_dir(&root).ok().into_iter().flatten().filter_map(Result::ok)
                    .filter_map(|entry|entry.file_name().to_str().and_then(|n|n.strip_prefix("live-")?.strip_suffix(".request")?.parse::<u64>().ok()))
                    .filter(|&id|id>seen).max();
                if let Some(id)=id.filter(|&id|id>seen) {
                    let result=std::fs::metadata(root.join(format!("live-{id}.ini"))).map_err(|e|e.to_string())
                        .and_then(|m|if m.len()>2_000_000 {Err("Live INI exceeds size limit".into())}else{Ok(())})
                        .and_then(|_|std::fs::read_to_string(root.join(format!("live-{id}.ini"))).map_err(|e|e.to_string()))
                        .and_then(|s|Evolution::parse(&s));
                    match tx.try_send((id,result)) {Ok(())=>seen=id,Err(mpsc::TrySendError::Full(_))=>{},Err(_)=>break}
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        });
    }
}
fn apply(mut config:ResMut<LiveConfig>,mut play:Option<ResMut<crate::gameplay::Play>>,
    generation:Res<crate::level_switch::LevelGeneration>,mut applied:Local<Option<u64>>) {
    let Some(ref mut play)=play else{return;};
    // Setup can replace Play after PreUpdate. Each fresh gameplay instance
    // receives the last acknowledged live profile before its following tick.
    let identity=generation.0;
    if play.is_added() || *applied!=Some(identity) {
        if let Some(next)=config.latest.as_ref(){play.svc.bomb_evo.apply_live_configuration(next);}
        *applied=Some(identity);
    }
    config.apply(&mut play.svc.bomb_evo);
}
impl LiveConfig {
    pub(crate) fn apply(&mut self,e:&mut Evolution) {
        let message=self.incoming.lock().ok().and_then(|rx|rx.try_recv().ok());
        let Some((id,result))=message else{return;};
        let result=result.map(|next|{e.apply_live_configuration(&next);self.latest=Some(next);});
        let _=self.replies.try_send((id,result));
    }
}
