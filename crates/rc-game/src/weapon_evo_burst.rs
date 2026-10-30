//! Optional fresh aim for direct burst shots. Only the launch vector changes;
//! projectile updates, native speed, lifetime, collision and damage remain native.
//! Callers supply their chosen target, preserving decoys and acquisition rules.
use crate::moby_runtime::MobyId;
use crate::moby_update::services::World;

pub fn enabled(w:&World,owner:MobyId)->bool {
    w.svc.bomb_evo.world_settings.enemy_burst_tracking && w.svc.game_mode==0
        && w.hero.health>0 && w.hero.group!=0x18 && w.hero.state!=0x72
        && !(w.svc.level==12 && w.hero.state==crate::hero::scripted::MOUNTED)
        && w.table.mobys.get(owner).is_some_and(|m|m.state<0x80
            && (crate::moby_update::real_hp::donor_class(m.o_class) || matches!(m.o_class,255|1274))
            && !crate::weapon_evo::world::boss(m.o_class))
}

pub fn velocity(w:&World,owner:MobyId,target:Option<MobyId>,origin:[f32;4],native:[f32;4])->[f32;4] {
    if !enabled(w,owner){return native;}
    let Some(id)=target else{return native;};
    let Some(m)=w.table.mobys.get(id).filter(|m|m.state<0x80) else{return native;};
    let aim=if Some(id)==w.hero_moby {w.hero.body_point.map(|p|p.to_f32())}
        else {[m.position[0],m.position[1],m.position[2]+0.6,0.]};
    redirected(origin,aim,native)
}
fn redirected(origin:[f32;4],aim:[f32;4],native:[f32;4])->[f32;4] {
    if !origin[..3].iter().chain(&aim[..3]).chain(&native[..3]).all(|v|v.is_finite()){return native;}
    let d=[aim[0]-origin[0],aim[1]-origin[1],aim[2]-origin[2]];
    let length=d.iter().map(|v|v*v).sum::<f32>().sqrt();
    let speed=native[..3].iter().map(|v|v*v).sum::<f32>().sqrt();
    if length<=0.0001||speed<=0.0001{return native;}
    [d[0]*speed/length,d[1]*speed/length,d[2]*speed/length,native[3]]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launches_reaim_without_changing_speed_or_previous_shots() {
        let origin=[1.,2.,3.,0.];let native=[3.,4.,0.,7.];
        let first=redirected(origin,[1.,12.,3.,0.],native);
        let second=redirected(origin,[1.,2.,13.,0.],native);
        assert_eq!(first,[0.,5.,0.,7.]);assert_eq!(second,[0.,0.,5.,7.]);
        assert_eq!(first,[0.,5.,0.,7.]);
        assert_eq!(redirected(origin,origin,native),native);
        assert_eq!(redirected(origin,[f32::NAN,0.,0.,0.],native),native);
    }
}
