"""Launcher-only localization. INI keys, save data and balance stay language-neutral.

Message rows are EN | IT | FR | DE | ES; field/help rows use a stable
INI key in the first column. English field help remains in schema.py. Weapon names use the original 2002
localizations; see docs/mod/LAUNCHER_V55.md for references and extension rules.
Off/On are deliberately identical in every language, as requested by the author.
"""
from __future__ import annotations
import re

LANGUAGES = {'en':'English', 'it':'Italiano', 'fr':'Français', 'de':'Deutsch', 'es':'Español'}
_language = 'en'

def set_language(language):
    global _language
    _language = language if language in LANGUAGES else 'en'

def language(): return _language

def rows(text):
    result = {}
    for line in text.strip().splitlines():
        if not line.strip() or line.startswith('#'): continue
        parts=[part.replace(r'\n','\n') for part in line.split('|')]
        if len(parts)!=5: raise ValueError('Invalid translation row: '+line)
        if parts[0].endswith(' '):parts=[part if part.endswith(' ') else part+' ' for part in parts]
        result[parts[0]]=dict(zip(LANGUAGES,parts))
    return result

WEAPON_NAMES = rows('''
Bomb Glove|Guanto Bomba|Gant à Bombes|Bomben-Handschuh|Guante bombas
Pyrocitor|Inceneritore|Pyrolanceur|Pyrozitor|Piromátic
Blaster|Blastatore|Plasmo-Mitrail|Blaster|Desintegrador
Glove of Doom|Guanto del Fato|Gant Ravageur|Todes-Handschuh|Guante perdición
Suck Cannon|Cannone a risucchio|Aspiro-Canon|Saug-Kanone|Cañón de succión
Taunter|Provocatore|Attrape-Nigaud|Verspötter|Burlófono
Mine Glove|Guanto Mina|Gant à Mines|Minen-Handschuh|Guante minas
Devastator|Devastatore|Dévastateur|Verwüster|Devastador
Visibomb Gun|Pistola Visibomba|Canon Visibombe|Visibombenwerfer|Visicañón
Walloper|Colpitore|Repousseur|Hieb|Guantazo
Decoy Glove|Guanto Esca|Gant à Leurres|Köder-Handschuh|Guante señuelo
Drone Device|Scudo Drone|Dronomatic|Abwehr-Drohnen-Handschuh|Guante defensor
Tesla Claw|Artiglio Tesla|Griffeur Tesla|Stromklaue|Garra Tesla
Morph-o-Ray|Morforaggio|Morpho-Rayon|Wandlerstrahl|Rayo mórfico
R.Y.N.O.|M.I.P.S.|T.E.L.T.|R.I.D.A.|T.A.U.N.
OmniWrench|Onnichiave 8000|Super Clé 8000|Omni-Schlüssel 8000|Omnillave 8000
''')

TEXT = rows(r'''
Language|Lingua|Langue|Sprache|Idioma
Launcher|Launcher|Lanceur|Launcher|Lanzador
Preset|Preset|Préréglage|Profil|Preajuste
Save|Salva|Enregistrer|Speichern|Guardar
Save as…|Salva con nome…|Enregistrer sous…|Speichern unter…|Guardar como…
Remove|Rimuovi|Supprimer|Entfernen|Eliminar
Custom|Personalizzato|Personnalisé|Benutzerdefiniert|Personalizado
Ready.|Pronto.|Prêt.|Bereit.|Listo.
Default configuration (weapons.ini)|Configurazione predefinita (weapons.ini)|Configuration par défaut (weapons.ini)|Standardkonfiguration (weapons.ini)|Configuración predeterminada (weapons.ini)
Launch settings|Impostazioni di avvio|Paramètres de lancement|Startoptionen|Opciones de inicio
Launcher settings|Impostazioni del launcher|Paramètres du lanceur|Launcher-Einstellungen|Opciones del lanzador
Import INI|Importa INI|Importer INI|INI importieren|Importar INI
Export INI|Esporta INI|Exporter INI|INI exportieren|Exportar INI
Global Settings|Impostazioni generali|Paramètres généraux|Allgemeine Einstellungen|Ajustes generales
Find setting:|Cerca impostazione:|Chercher un paramètre :|Einstellung suchen:|Buscar ajuste:
Save and launch|Salva e avvia|Enregistrer et lancer|Speichern und starten|Guardar e iniciar
Open log|Apri log|Ouvrir le journal|Protokoll öffnen|Abrir registro
Game closed|Gioco chiuso|Jeu fermé|Spiel geschlossen|Juego cerrado
Game running|Gioco in esecuzione|Jeu en cours|Spiel läuft|Juego en ejecución
Game closed (exit code {code}). |Gioco chiuso (codice {code}). |Jeu fermé (code {code}). |Spiel geschlossen (Code {code}). |Juego cerrado (código {code}).
Invalid configuration|Configurazione non valida|Configuration invalide|Ungültige Konfiguration|Configuración no válida
Preset loaded: |Preset caricato: |Préréglage chargé : |Profil geladen: |Preajuste cargado:
Initial configuration: |Configurazione iniziale: |Configuration initiale : |Ausgangskonfiguration: |Configuración inicial:
First launch: choose your extracted game data folder.|Primo avvio: scegli la cartella dei dati estratti del gioco.|Premier lancement : choisissez le dossier des données extraites du jeu.|Erster Start: Ordner mit extrahierten Spieldaten auswählen.|Primer inicio: elige la carpeta de datos extraídos del juego.
{profile}  ·  {count} unsaved changes  ·  configuration values|{profile}  ·  {count} modifiche non salvate  ·  valori di configurazione|{profile}  ·  {count} modifications non enregistrées  ·  configuration|{profile}  ·  {count} ungespeicherte Änderungen  ·  Konfigurationswerte|{profile}  ·  {count} cambios sin guardar  ·  valores de configuración
Load a valid weapons.ini before continuing.|Carica un weapons.ini valido prima di continuare.|Chargez un weapons.ini valide avant de continuer.|Vor dem Fortfahren eine gültige weapons.ini laden.|Carga un weapons.ini válido antes de continuar.
Load a valid weapons.ini to get started.|Carica un weapons.ini valido per iniziare.|Chargez un weapons.ini valide pour commencer.|Zum Start eine gültige weapons.ini laden.|Carga un weapons.ini válido para empezar.
Unsaved changes|Modifiche non salvate|Modifications non enregistrées|Ungespeicherte Änderungen|Cambios sin guardar
Save changes before switching configurations?|Salvare le modifiche prima di cambiare configurazione?|Enregistrer les modifications avant de changer de configuration ?|Änderungen vor dem Konfigurationswechsel speichern?|¿Guardar los cambios antes de cambiar de configuración?
Loaded: |Caricato: |Chargé : |Geladen: |Cargado:
Default configuration saved: |Configurazione predefinita salvata: |Configuration par défaut enregistrée : |Standardkonfiguration gespeichert: |Configuración predeterminada guardada:
Preset name:|Nome del preset:|Nom du préréglage :|Profilname:|Nombre del preajuste:
Choose a name other than the default configuration label.|Scegli un nome diverso da quello della configurazione predefinita.|Choisissez un autre nom que celui de la configuration par défaut.|Einen anderen Namen als die Standardkonfiguration wählen.|Elige un nombre distinto al de la configuración predeterminada.
Existing preset|Preset esistente|Préréglage existant|Vorhandenes Profil|Preajuste existente
Update {name}? The previous version will be backed up.|Aggiornare {name}? Verrà salvata una copia della versione precedente.|Mettre à jour {name} ? La version précédente sera sauvegardée.|{name} aktualisieren? Die bisherige Version wird gesichert.|¿Actualizar {name}? Se guardará una copia de la versión anterior.
Update this preset? The previous version will be backed up.|Aggiornare questo preset? Verrà salvata una copia della versione precedente.|Mettre à jour ce préréglage ? La version précédente sera sauvegardée.|Dieses Profil aktualisieren? Die bisherige Version wird gesichert.|¿Actualizar este preajuste? Se guardará una copia de la versión anterior.
Preset saved: |Preset salvato: |Préréglage enregistré : |Profil gespeichert: |Preajuste guardado:
The default configuration cannot be removed.|La configurazione predefinita non può essere rimossa.|La configuration par défaut ne peut pas être supprimée.|Die Standardkonfiguration kann nicht entfernt werden.|No se puede eliminar la configuración predeterminada.
Preset moved to backup: |Preset spostato nel backup: |Préréglage déplacé vers la sauvegarde : |Profil in Sicherung verschoben: |Preajuste movido a la copia de seguridad:
Import weapons.ini|Importa weapons.ini|Importer weapons.ini|weapons.ini importieren|Importar weapons.ini
All files|Tutti i file|Tous les fichiers|Alle Dateien|Todos los archivos
Import as preset|Importa come preset|Importer comme préréglage|Als Profil importieren|Importar como preajuste
New preset name:|Nome del nuovo preset:|Nom du nouveau préréglage :|Neuer Profilname:|Nombre del nuevo preajuste:
This name is reserved for the default configuration.|Questo nome è riservato alla configurazione predefinita.|Ce nom est réservé à la configuration par défaut.|Dieser Name ist für die Standardkonfiguration reserviert.|Este nombre está reservado para la configuración predeterminada.
INI imported: |INI importato: |INI importé : |INI importiert: |INI importado:
Export preset INI|Esporta preset INI|Exporter le préréglage INI|INI-Profil exportieren|Exportar preajuste INI
Exported: |Esportato: |Exporté : |Exportiert: |Exportado:
Use Save to update weapons.ini, or choose another export filename.|Usa Salva per aggiornare weapons.ini, oppure scegli un altro nome per l'esportazione.|Utilisez Enregistrer pour mettre à jour weapons.ini, ou choisissez un autre nom d'exportation.|Mit Speichern weapons.ini aktualisieren oder einen anderen Exportnamen wählen.|Usa Guardar para actualizar weapons.ini o elige otro nombre de exportación.
Restore saved value|Ripristina valore salvato|Rétablir la valeur enregistrée|Gespeicherten Wert wiederherstellen|Restaurar valor guardado
Copy this value to the other grades|Copia questo valore negli altri gradi|Copier cette valeur aux autres grades|Wert in andere Stufen kopieren|Copiar este valor a los demás grados
Original color (default)|Colore originale (default)|Couleur d'origine (default)|Originalfarbe (default)|Color original (default)
Value copied to all five grades: |Valore copiato in tutti i cinque gradi: |Valeur copiée aux cinq grades : |Wert in alle fünf Stufen kopiert: |Valor copiado a los cinco grados:
Base settings|Impostazioni base|Paramètres de base|Grundeinstellungen|Ajustes básicos
Standard settings|Impostazioni standard|Paramètres standard|Standardeinstellungen|Ajustes estándar
XP bar and HUD|Barra XP e HUD|Barre d'XP et HUD|XP-Leiste und HUD|Barra de XP y HUD
Experimental settings|Impostazioni sperimentali|Paramètres expérimentaux|Experimentelle Einstellungen|Ajustes experimentales
Ammo-Survival Mode|Ammo-Survival Mode|Ammo-Survival Mode|Ammo-Survival Mode|Ammo-Survival Mode
New games start with zero Bomb Glove ammo. Existing save ammo is preserved.|Le nuove partite iniziano senza munizioni del Guanto Bomba. Le munizioni dei salvataggi esistenti vengono conservate.|Les nouvelles parties commencent sans munitions du Gant à Bombes. Les sauvegardes existantes conservent leurs munitions.|Neue Spiele beginnen ohne Munition für den Bomben-Handschuh. Vorhandene Spielstände behalten ihre Munition.|Las partidas nuevas empiezan sin munición del Guante bombas. Las partidas existentes conservan su munición.
Permanent weapon unlocks (collected normal bolts)|Sblocco permanente delle armi (bolt normali raccolti)|Déblocage permanent des armes (boulons ordinaires collectés)|Dauerhafte Waffenfreischaltung (gesammelte normale Bolts)|Desbloqueos permanentes (guitones normales recogidos)
Ammo crate pack probabilities|Probabilità dei caricatori nelle casse munizioni|Probabilités des packs dans les caisses de munitions|Wahrscheinlichkeiten für Munitionspakete in Kisten|Probabilidades de cargadores en cajas de munición
Inputs are normalized to 100%. Set 0 to exclude an ammo type.|I valori vengono normalizzati al 100%. Imposta 0 per escludere un tipo di munizioni.|Les valeurs sont normalisées à 100 %. Mettez 0 pour exclure un type de munitions.|Die Werte werden auf 100 % normiert. Mit 0 einen Munitionstyp ausschließen.|Los valores se normalizan al 100 %. Pon 0 para excluir un tipo de munición.
XP progression|Progressione XP|Progression d'XP|XP-Fortschritt|Progresión de XP
Game ID: |ID di gioco: |ID du jeu : |Spiel-ID: |ID del juego:
  ·  XP required for each upgrade|  ·  XP necessari per ogni potenziamento|  ·  XP requis pour chaque amélioration|  ·  XP pro Aufrüstung|  ·  XP necesarios para cada mejora
V{grade}  /  Grade {grade}|V{grade}  /  Grado {grade}|V{grade}  /  Grade {grade}|V{grade}  /  Stufe {grade}|V{grade}  /  Grado {grade}
Maximum grade|Grado massimo|Grade maximal|Höchste Stufe|Grado máximo
No settings for this grade in the INI.|Nessuna impostazione per questo grado nell'INI.|Aucun paramètre pour ce grade dans l'INI.|Keine Einstellungen für diese Stufe in der INI.|No hay ajustes para este grado en el INI.
All five grades are on this page. Scroll to see them all. Right-click a value to copy it to the other grades.|Questa pagina contiene i cinque gradi. Scorri per vederli tutti. Clic destro su un valore per copiarlo negli altri gradi.|Les cinq grades sont sur cette page. Faites défiler pour tous les voir. Clic droit sur une valeur pour la copier aux autres grades.|Alle fünf Stufen stehen auf dieser Seite. Scrollen, um alle zu sehen. Rechtsklick auf einen Wert kopiert ihn in andere Stufen.|Los cinco grados están en esta página. Desplázate para verlos. Clic derecho en un valor para copiarlo a los demás grados.
Global XP, HUD and level-up settings. In-game weapon names follow the game language.|Impostazioni generali di XP, HUD e potenziamenti. I nomi delle armi in gioco seguono la lingua del gioco.|Paramètres généraux d'XP, de HUD et d'amélioration. Les noms des armes en jeu suivent la langue du jeu.|Allgemeine XP-, HUD- und Aufrüstungsoptionen. Waffennamen im Spiel folgen der Spielsprache.|Opciones generales de XP, HUD y mejoras. Los nombres de armas en el juego siguen el idioma del juego.
Actual chance per pack: |Probabilità effettiva per caricatore: |Probabilité réelle par pack : |Tatsächliche Chance pro Paket: |Probabilidad real por cargador:
Enter valid probabilities; at least one ammo type must be above zero.|Inserisci probabilità valide; almeno un tipo di munizioni deve superare zero.|Saisissez des probabilités valides ; au moins un type de munitions doit dépasser zéro.|Gültige Wahrscheinlichkeiten eingeben; mindestens ein Munitionstyp muss über null liegen.|Introduce probabilidades válidas; al menos un tipo de munición debe superar cero.
Extracted game data|Dati estratti del gioco|Données extraites du jeu|Extrahierte Spieldaten|Datos extraídos del juego
Save folder (leave empty to use the default)|Cartella salvataggi (vuota per usare quella predefinita)|Dossier des sauvegardes (vide pour utiliser celui par défaut)|Spielstandordner (leer für Standardordner)|Carpeta de partidas (vacía para usar la predeterminada)
Modded ReRAC executable|Eseguibile ReRAC modificato|Exécutable ReRAC modifié|Modifizierte ReRAC-Programmdatei|Ejecutable ReRAC modificado
Browse…|Sfoglia…|Parcourir…|Durchsuchen…|Examinar…
Choose the folder containing toc.bin and extract-info.json.\nThe included modded game is selected automatically.\nLeave the save folder empty to keep using your existing saves.|Scegli la cartella con toc.bin ed extract-info.json.\nIl gioco modificato incluso viene selezionato automaticamente.\nLascia vuota la cartella salvataggi per continuare a usare quelli esistenti.|Choisissez le dossier contenant toc.bin et extract-info.json.\nLe jeu modifié inclus est sélectionné automatiquement.\nLaissez le dossier de sauvegarde vide pour conserver vos sauvegardes actuelles.|Ordner mit toc.bin und extract-info.json wählen.\nDas enthaltene modifizierte Spiel wird automatisch gewählt.\nSpielstandordner leer lassen, um vorhandene Spielstände weiterzuverwenden.|Elige la carpeta con toc.bin y extract-info.json.\nEl juego modificado incluido se selecciona automáticamente.\nDeja vacía la carpeta de partidas para seguir usando las existentes.
Save settings|Salva impostazioni|Enregistrer les paramètres|Einstellungen speichern|Guardar ajustes
Choose rerac.exe|Scegli rerac.exe|Choisir rerac.exe|rerac.exe auswählen|Elegir rerac.exe
Executable|Eseguibile|Exécutable|Programmdatei|Ejecutable
Choose the game data folder in Launch settings.|Scegli la cartella dei dati del gioco nelle impostazioni di avvio.|Choisissez le dossier des données du jeu dans les paramètres de lancement.|Spieldatenordner in den Startoptionen wählen.|Elige la carpeta de datos del juego en las opciones de inicio.
Checking the game and configuration…|Verifica del gioco e della configurazione…|Vérification du jeu et de la configuration…|Spiel und Konfiguration werden geprüft…|Comprobando el juego y la configuración…
Started: |Avviato: |Lancé : |Gestartet: |Iniciado:
 · waiting for the game to confirm the configuration.| · in attesa della conferma della configurazione da parte del gioco.| · en attente de confirmation de la configuration par le jeu.| · Bestätigung der Konfiguration durch das Spiel wird erwartet.| · esperando que el juego confirme la configuración.
Configuration confirmed by the game: |Configurazione confermata dal gioco: |Configuration confirmée par le jeu : |Konfiguration vom Spiel bestätigt: |Configuración confirmada por el juego:
Configuration confirmed.|Configurazione confermata.|Configuration confirmée.|Konfiguration bestätigt.|Configuración confirmada.
Configuration loading has not been confirmed. Open the log.|Il caricamento della configurazione non è stato confermato. Apri il log.|Le chargement de la configuration n'a pas été confirmé. Ouvrez le journal.|Das Laden der Konfiguration wurde nicht bestätigt. Protokoll öffnen.|No se ha confirmado la carga de la configuración. Abre el registro.
Reading log: |Lettura log: |Lecture du journal : |Protokoll wird gelesen: |Leyendo registro:
No log available. Start the game from the launcher.|Nessun log disponibile. Avvia il gioco dal launcher.|Aucun journal disponible. Lancez le jeu depuis le lanceur.|Kein Protokoll verfügbar. Spiel über den Launcher starten.|No hay registro disponible. Inicia el juego desde el lanzador.
Wait for startup to finish before closing the launcher.|Attendi la fine dell'avvio prima di chiudere il launcher.|Attendez la fin du lancement avant de fermer le lanceur.|Vor dem Schließen des Launchers das Startende abwarten.|Espera a que termine el inicio antes de cerrar el lanzador.
 Range: {low} … {high}.| Intervallo: {low} … {high}.| Plage : {low} … {high}.| Bereich: {low} … {high}.| Intervalo: {low} … {high}.
Technical details:|Dettagli tecnici:|Détails techniques :|Technische Details:|Detalles técnicos:
Could not complete the operation.|Impossibile completare l'operazione.|Impossible de terminer l'opération.|Der Vorgang konnte nicht abgeschlossen werden.|No se pudo completar la operación.
Notification sound|Suono di notifica|Son de notification|Benachrichtigungston|Sonido de notificación
Companion only|Solo compagno|Compagnon seul|Nur Begleiter|Solo acompañante
Companion and drones|Compagno e droni|Compagnon et drones|Begleiter und Drohnen|Acompañante y drones
Companion with Sonic Summoner|Compagno con Sonic Summoner|Compagnon avec l'Appâte-Sandmouse|Begleiter mit Schall-Summoner|Acompañante con el Invocador sónico
{weapon} unlock (collected bolts)|Sblocco {weapon} (bolt raccolti)|Déblocage de {weapon} (boulons collectés)|{weapon} freischalten (gesammelte Bolts)|Desbloqueo de {weapon} (guitones recogidos)
{weapon} ammo chance (%)|Probabilità munizioni {weapon} (%)|Probabilité de munitions {weapon} (%)|Munitionschance {weapon} (%)|Probabilidad de munición de {weapon} (%)
XP from V{grade} to V{next_grade}|XP da V{grade} a V{next_grade}|XP de V{grade} à V{next_grade}|XP von V{grade} auf V{next_grade}|XP de V{grade} a V{next_grade}
''')

def tr(source, **values):
    result=TEXT.get(source,{}).get(_language,source)
    return result.format(**values) if values else result

def weapon_name(key):
    from schema import WEAPONS
    return 'OmniWrench 8000' if key=='wrench' and _language=='en' else WEAPON_NAMES[WEAPONS[key][1]][_language]

def boolean_display(value):
    return 'On' if str(value).lower() in ('true','yes','1') else 'Off'

def boolean_value(display):
    if display not in ('Off','On'): raise ValueError('Expected Off or On')
    return 'true' if display=='On' else 'false'

# Key-based labels avoid translating technical identifiers or saved values.
FIELD_LABELS = rows('''
id|ID di gioco|ID du jeu|Spiel-ID|ID del juego
max_ammo|Munizioni massime|Munitions maximales|Maximale Munition|Munición máxima
experimental_max_ammo|Munizioni massime sperimentali|Munitions maximales expérimentales|Experimentelle maximale Munition|Munición máxima experimental
damage_hp|Danno per colpo (HP)|Dégâts par coup (PV)|Schaden pro Treffer (HP)|Daño por impacto (PV)
damage_per_second|Danno al secondo (HP/s)|Dégâts par seconde (PV/s)|Schaden pro Sekunde (HP/s)|Daño por segundo (PV/s)
chicken_damage_hp|Danno esplosione pollo (HP)|Dégâts d'explosion du poulet (PV)|Hühnerexplosionsschaden (HP)|Daño de explosión del pollo (PV)
damage_multiplier|Moltiplicatore danni precedente|Ancien multiplicateur de dégâts|Alter Schadensmultiplikator|Multiplicador de daño anterior
gold|Comportamento e texture d'oro (precedente)|Comportement et texture en or (ancien)|Gold-Verhalten und -Textur (alt)|Comportamiento y textura de oro (anterior)
gold_effects|Comportamento d'oro|Comportement en or|Gold-Verhalten|Comportamiento de oro
gold_texture|Texture d'oro|Texture en or|Gold-Textur|Textura de oro
damage_interval_ticks|Intervallo registrazione danni|Intervalle d'enregistrement des dégâts|Intervall der Schadenserfassung|Intervalo de registro de daño
fire_interval_ticks|Intervallo di fuoco|Intervalle de tir|Feuerintervall|Intervalo de disparo
ammo_interval_ticks|Intervallo consumo munizioni|Intervalle de consommation des munitions|Munitionsverbrauchsintervall|Intervalo de consumo de munición
projectile_speed_multiplier|Velocità proiettile|Vitesse du projectile|Projektilgeschwindigkeit|Velocidad del proyectil
projectile_range_multiplier|Distanza massima proiettile|Portée du projectile|Projektilreichweite|Alcance del proyectil
projectile_color|Colore proiettile|Couleur du projectile|Projektilfarbe|Color del proyectil
trail_color|Colore scia|Couleur de la traînée|Schweiffarbe|Color de la estela
impact_color|Colore impatto|Couleur de l'impact|Aufprallfarbe|Color del impacto
missile_speed_multiplier|Velocità missile|Vitesse du missile|Raketengeschwindigkeit|Velocidad del misil
missile_range_multiplier|Distanza massima missile|Portée du missile|Raketenreichweite|Alcance del misil
homing_strength_multiplier|Efficacia inseguimento|Puissance de guidage|Zielsuchstärke|Eficacia del guiado
explosion_radius_multiplier|Raggio danni esplosione|Rayon des dégâts d'explosion|Explosionsschadensradius|Radio de daño de explosión
explosion_visual_size_multiplier|Dimensione visiva esplosione|Taille visuelle de l'explosion|Sichtbare Explosionsgröße|Tamaño visual de explosión
explosion_color|Colore esplosione|Couleur de l'explosion|Explosionsfarbe|Color de explosión
throw_range|Distanza di lancio|Portée du lancer|Wurfreichweite|Alcance de lanzamiento
auto_lock_range|Distanza aggancio automatico|Portée du verrouillage automatique|Automatische Zielerfassungsreichweite|Alcance de fijación automática
throw_range_multiplier|Distanza di lancio|Portée du lancer|Wurfreichweite|Alcance de lanzamiento
trajectory_height|Altezza traiettoria|Hauteur de la trajectoire|Flugbahnhöhe|Altura de la trayectoria
trajectory_height_multiplier|Altezza traiettoria|Hauteur de la trajectoire|Flugbahnhöhe|Altura de la trayectoria
explosion_radius|Raggio esplosione|Rayon de l'explosion|Explosionsradius|Radio de explosión
cooldown_seconds|Tempo tra i lanci (secondi)|Temps entre lancers (secondes)|Zeit zwischen Würfen (Sekunden)|Tiempo entre lanzamientos (segundos)
flame_range|Lunghezza fiamma|Longueur des flammes|Flammenlänge|Longitud de la llama
flame_width|Larghezza fiamma|Largeur des flammes|Flammenbreite|Anchura de la llama
flame_core_color|Colore nucleo fiamma|Couleur du cœur des flammes|Flammenkernfarbe|Color del núcleo de la llama
flame_outer_color|Colore esterno fiamma|Couleur extérieure des flammes|Äußere Flammenfarbe|Color exterior de la llama
burn_flame_color|Colore fiamma sui nemici|Couleur des flammes sur les ennemis|Brandfarbe auf Gegnern|Color de llamas en enemigos
light_color|Colore luce|Couleur de la lumière|Lichtfarbe|Color de la luz
capacity|Capacità di risucchio|Capacité d'aspiration|Saugkapazität|Capacidad de succión
suck_range_multiplier|Distanza di risucchio|Portée d'aspiration|Saugreichweite|Alcance de succión
shot_damage_multiplier|Moltiplicatore danni precedente|Ancien multiplicateur de dégâts|Alter Schadensmultiplikator|Multiplicador de daño anterior
duration_multiplier|Durata missile guidato|Durée du missile guidé|Dauer der gelenkten Rakete|Duración del misil guiado
turn_multiplier|Capacità di virata|Capacité de virage|Wendigkeit|Capacidad de giro
invulnerable_while_guiding|Invulnerabilità durante la guida|Invulnérabilité pendant le guidage|Unverwundbar beim Lenken|Invulnerabilidad al guiar
lure_range|Distanza di attrazione|Portée d'attraction|Anlockreichweite|Alcance de atracción
lure_cone_degrees|Cono di attrazione (gradi)|Cône d'attraction (degrés)|Anlockkegel (Grad)|Cono de atracción (grados)
xp_attract_fraction|Frazione HP per XP da attrazione|Fraction des PV pour l'XP d'attraction|HP-Anteil für Anlock-XP|Fracción de PV para XP por atracción
stun_range|Raggio stordimento|Portée d'étourdissement|Betäubungsreichweite|Alcance de aturdimiento
stun_duration_ticks|Durata stordimento|Durée d'étourdissement|Betäubungsdauer|Duración del aturdimiento
stun_efficiency_percent|Probabilità stordimento (%)|Chance d'étourdissement (%)|Betäubungschance (%)|Probabilidad de aturdimiento (%)
damage_range|Raggio danni|Portée des dégâts|Schadensreichweite|Alcance de daño
wave_speed_multiplier|Velocità onda visiva|Vitesse de l'onde visuelle|Geschwindigkeit der sichtbaren Welle|Velocidad de la onda visual
wave_size_multiplier|Dimensione onda visiva|Taille de l'onde visuelle|Größe der sichtbaren Welle|Tamaño de la onda visual
wave_range_multiplier|Distanza onda visiva|Portée de l'onde visuelle|Reichweite der sichtbaren Welle|Alcance de la onda visual
wave_interval_ticks|Intervallo onde visive|Intervalle des ondes visuelles|Intervall der sichtbaren Wellen|Intervalo de ondas visuales
wave_alpha|Opacità onda visiva|Opacité de l'onde visuelle|Deckkraft der sichtbaren Welle|Opacidad de la onda visual
wave_color|Colore onda visiva|Couleur de l'onde visuelle|Farbe der sichtbaren Welle|Color de la onda visual
activation_range_horizontal|Distanza attivazione orizzontale|Portée d'activation horizontale|Horizontale Aktivierungsreichweite|Alcance horizontal de activación
activation_range_vertical|Distanza attivazione verticale|Portée d'activation verticale|Vertikale Aktivierungsreichweite|Alcance vertical de activación
knockback_multiplier|Forza respingimento|Force de recul|Rückstoßstärke|Fuerza de empuje
hit_radius_multiplier|Raggio del colpo|Rayon du coup|Trefferradius|Radio del golpe
electric_fx_size_multiplier|Dimensione effetti elettrici|Taille des effets électriques|Größe der elektrischen Effekte|Tamaño de efectos eléctricos
gold_electric_fx|Effetti elettrici d'oro|Effets électriques en or|Gold-Elektroeffekte|Efectos eléctricos de oro
air_dash_enabled|Scatto in aria|Ruée aérienne|Luftsprint|Impulso aéreo
electric_core_color|Colore nucleo elettrico|Couleur du cœur électrique|Farbe des elektrischen Kerns|Color del núcleo eléctrico
electric_glow_color|Colore bagliore elettrico|Couleur de la lueur électrique|Elektrische Leuchtfarbe|Color del brillo eléctrico
fist_glow_color|Colore bagliore pugno|Couleur de la lueur du poing|Faustleuchtfarbe|Color del brillo del puño
target_range_multiplier|Distanza bersaglio|Portée des cibles|Zielreichweite|Alcance del objetivo
lock_cone_multiplier|Ampiezza cono di aggancio|Largeur du cône de verrouillage|Breite des Zielerfassungskegels|Anchura del cono de fijación
beam_core_color|Colore nucleo raggio|Couleur du cœur du rayon|Strahlkernfarbe|Color del núcleo del rayo
beam_glow_color|Colore bagliore raggio|Couleur de la lueur du rayon|Strahlleuchtfarbe|Color del brillo del rayo
secondary_core_color|Colore nucleo raggio secondario|Couleur du cœur du rayon secondaire|Kernfarbe des zweiten Strahls|Color del núcleo del rayo secundario
secondary_glow_color|Colore bagliore raggio secondario|Couleur de la lueur du rayon secondaire|Leuchtfarbe des zweiten Strahls|Color del brillo del rayo secundario
arc_core_color|Colore nucleo arco elettrico|Couleur du cœur de l'arc électrique|Lichtbogenkernfarbe|Color del núcleo del arco eléctrico
arc_glow_color|Colore bagliore arco elettrico|Couleur de la lueur de l'arc électrique|Lichtbogenleuchtfarbe|Color del brillo del arco eléctrico
claw_core_color|Colore nucleo artiglio|Couleur du cœur de la griffe|Klauenkernfarbe|Color del núcleo de la garra
claw_glow_color|Colore bagliore artiglio|Couleur de la lueur de la griffe|Klauenleuchtfarbe|Color del brillo de la garra
spark_color|Colore scintille|Couleur des étincelles|Funkenfarbe|Color de chispas
spark_core_color|Colore nucleo scintille|Couleur du cœur des étincelles|Funkenkernfarbe|Color del núcleo de chispas
weapon_glow_color|Colore bagliore arma|Couleur de la lueur de l'arme|Waffenleuchtfarbe|Color del brillo del arma
robot_count|Robot per lancio|Robots par lancer|Roboter pro Wurf|Robots por lanzamiento
robot_max_alive|Numero massimo robot attivi|Nombre maximal de robots actifs|Maximal aktive Roboter|Máximo de robots activos
robot_detection_range|Distanza rilevamento robot|Portée de détection des robots|Robotererkennungsreichweite|Alcance de detección de robots
fire_in_air|Fuoco in aria|Tir en l'air|In der Luft feuern|Disparar en el aire
morph_speed_multiplier|Velocità trasformazione|Vitesse de transformation|Verwandlungsgeschwindigkeit|Velocidad de transformación
beam_range_multiplier|Distanza raggio|Portée du rayon|Strahlreichweite|Alcance del rayo
explode_small_chickens|Esplosione polli piccoli|Explosion des petits poulets|Kleine Hühner explodieren|Explosión de pollos pequeños
chicken_damage_multiplier|Moltiplicatore danni pollo precedente|Ancien multiplicateur des dégâts du poulet|Alter Hühnerschadensmultiplikator|Multiplicador anterior de daño del pollo
chicken_explosion_radius_multiplier|Raggio esplosione pollo|Rayon d'explosion du poulet|Hühnerexplosionsradius|Radio de explosión del pollo
tube_color|Colore tubo|Couleur du tube|Rohrfarbe|Color del tubo
pulse_color|Colore impulso|Couleur de l'impulsion|Impulsfarbe|Color del pulso
muzzle_core_color|Colore nucleo bocca di fuoco|Couleur du cœur de la bouche|Mündungskernfarbe|Color del núcleo de la boca
muzzle_glow_color|Colore bagliore bocca di fuoco|Couleur de la lueur de la bouche|Mündungsleuchtfarbe|Color del brillo de la boca
tip_color|Colore punta raggio|Couleur du bout du rayon|Strahlspitzenfarbe|Color de la punta del rayo
tip_core_color|Colore nucleo punta raggio|Couleur du cœur du bout du rayon|Kernfarbe der Strahlspitze|Color del núcleo de la punta del rayo
missiles_per_salvo|Missili per raffica|Missiles par salve|Raketen pro Salve|Misiles por ráfaga
drone_count|Numero droni|Nombre de drones|Drohnenanzahl|Número de drones
detection_range_multiplier|Distanza rilevamento droni|Portée de détection des drones|Drohnenerkennungsreichweite|Alcance de detección de drones
drone_size_multiplier|Dimensione droni|Taille des drones|Drohnengröße|Tamaño de drones
blast_radius_multiplier|Raggio esplosione droni|Rayon d'explosion des drones|Drohnenexplosionsradius|Radio de explosión de drones
decoy_health_multiplier|Vita esca|PV du leurre|Ködergesundheit|Vida del señuelo
explosion_enabled|Esplosione attiva|Explosion activée|Explosion aktiviert|Explosión activada
explosion_damage_multiplier|Moltiplicatore danni esplosione precedente|Ancien multiplicateur des dégâts d'explosion|Alter Explosionsschadensmultiplikator|Multiplicador anterior de daño de explosión
xp_per_damage|XP per HP di danno|XP par PV de dégâts|XP pro Schadens-HP|XP por PV de daño
name_opacity_percent|Opacità nome arma (%)|Opacité du nom de l'arme (%)|Deckkraft des Waffennamens (%)|Opacidad del nombre del arma (%)
xp_bar_height_px|Altezza barra XP (pixel)|Hauteur de la barre d'XP (pixels)|Höhe der XP-Leiste (Pixel)|Altura de la barra de XP (píxeles)
xp_bar_horizontal_inset_px|Margine orizzontale nome (pixel)|Marge horizontale du nom (pixels)|Horizontaler Namensabstand (Pixel)|Margen horizontal del nombre (píxeles)
show_for_weapons_without_ammo|Mostra HUD per armi senza munizioni|Afficher le HUD des armes sans munitions|HUD für Waffen ohne Munition anzeigen|Mostrar HUD para armas sin munición
level_up_popup|Notifica potenziamento|Notification d'amélioration|Aufrüstungsbenachrichtigung|Notificación de mejora
level_up_popup_duration_seconds|Durata notifica (secondi)|Durée de notification (secondes)|Benachrichtigungsdauer (Sekunden)|Duración de notificación (segundos)
level_up_sound|Suono potenziamento|Son d'amélioration|Aufrüstungston|Sonido de mejora
xp_to_next|XP al grado successivo (precedente)|XP au grade suivant (ancien)|XP zur nächsten Stufe (alt)|XP al siguiente grado (anterior)
experimental_ammo_unlock_mode|Ammo-Survival Mode|Ammo-Survival Mode|Ammo-Survival Mode|Ammo-Survival Mode
cluster_bombs_enabled|Mini bombe sperimentali|Mini-bombes expérimentales|Experimentelle Streubomben|Minibombas experimentales
cluster_bomb_count|Mini bombe per esplosione|Mini-bombes par explosion|Minibomben pro Explosion|Minibombas por explosión
cluster_bomb_size_percent|Dimensione mini bombe (%)|Taille des mini-bombes (%)|Minibombengröße (%)|Tamaño de minibombas (%)
cluster_explosion_size_percent|Dimensione mini esplosione (%)|Taille des mini-explosions (%)|Mini-Explosionsgröße (%)|Tamaño de miniexplosión (%)
cluster_damage_percent|Danno mini bombe (%)|Dégâts des mini-bombes (%)|Minibombenschaden (%)|Daño de minibombas (%)
cluster_launch_speed|Velocità orizzontale mini bombe (unità/s)|Vitesse horizontale des mini-bombes (unités/s)|Horizontale Minibombengeschwindigkeit (Einheiten/s)|Velocidad horizontal de minibombas (unidades/s)
cluster_jump_speed|Velocità iniziale salto mini bombe (unità/s)|Vitesse de saut initiale des mini-bombes (unités/s)|Anfängliche Minibomben-Sprunggeschwindigkeit (Einheiten/s)|Velocidad inicial de salto de minibombas (unidades/s)
triple_flame_enabled|Tripla fiamma sperimentale|Triple flamme expérimentale|Experimentelle Dreifachflamme|Triple llama experimental
side_flame_left_angle_degrees|Angolo fiamma sinistra (gradi)|Angle de la flamme gauche (degrés)|Linker Flammenwinkel (Grad)|Ángulo de llama izquierda (grados)
side_flame_right_angle_degrees|Angolo fiamma destra (gradi)|Angle de la flamme droite (degrés)|Rechter Flammenwinkel (Grad)|Ángulo de llama derecha (grados)
held_lunge_duration_percent|Durata pugno tenuto (%)|Durée du coup maintenu (%)|Dauer des gehaltenen Schlags (%)|Duración del puñetazo mantenido (%)
crate_suction_enabled|Risucchio casse sperimentale|Aspiration de caisses expérimentale|Experimentelles Kistensaugen|Succión experimental de cajas
suck_wooden_crates|Consenti casse di legno|Autoriser les caisses en bois|Holzkisten zulassen|Permitir cajas de madera
suck_explosive_crates|Consenti casse esplosive|Autoriser les caisses explosives|Explosive Kisten zulassen|Permitir cajas explosivas
suck_steel_crates|Consenti casse d'acciaio|Autoriser les caisses en acier|Stahlkisten zulassen|Permitir cajas de acero
suck_ammo_crates|Consenti casse munizioni|Autoriser les caisses de munitions|Munitionskisten zulassen|Permitir cajas de munición
suck_nanotech_crates|Consenti casse Nanotech|Autoriser les caisses de Nanotech|Nanotech-Kisten zulassen|Permitir cajas de Nanotech
beam_size_percent|Dimensione raggio (%)|Taille du rayon (%)|Strahlgröße (%)|Tamaño del rayo (%)
dual_beam_enabled|Doppio raggio sperimentale|Double rayon expérimental|Experimenteller Doppelstrahl|Doble rayo experimental
homing_strength_multiplier|Efficacia inseguimento|Puissance de guidage|Zielsuchstärke|Eficacia del guiado
missile_max_range|Distanza massima missile (unità)|Portée maximale du missile (unités)|Maximale Raketenreichweite (Einheiten)|Alcance máximo del misil (unidades)
explosion_damage_radius|Raggio danni esplosione (unità)|Rayon des dégâts d'explosion (unités)|Explosionsschadensradius (Einheiten)|Radio de daño de explosión (unidades)
max_active_mines|Numero massimo mine attive|Nombre maximal de mines actives|Maximal aktive Minen|Máximo de minas activas
mine_lifetime_seconds|Durata mine (secondi)|Durée des mines (secondes)|Minendauer (Sekunden)|Duración de minas (segundos)
stun_launch_distance|Distanza spinta da stordimento|Distance de projection après étourdissement|Stoßweite bei Betäubung|Distancia de empuje por aturdimiento
grenade_launcher_enabled|Lanciagranate sperimentale|Lance-grenades expérimental|Experimenteller Granatwerfer|Lanzagranadas experimental
grenade_size_percent|Dimensione granata (%)|Taille des grenades (%)|Granatengröße (%)|Tamaño de granada (%)
grenade_arc_height|Altezza arco granata (unità)|Hauteur de l'arc des grenades (unités)|Granatenbogenhöhe (Einheiten)|Altura del arco de granada (unidades)
grenade_max_range|Distanza orizzontale granata (unità)|Portée horizontale des grenades (unités)|Horizontale Granatenreichweite (Einheiten)|Alcance horizontal de granada (unidades)
grenade_fire_interval_ticks|Intervallo lancio granate (tick)|Intervalle de tir des grenades (ticks)|Granatenfeuerintervall (Ticks)|Intervalo de lanzamiento de granadas (ticks)
grenade_damage_hp|Danno esplosione granata (HP)|Dégâts d'explosion des grenades (PV)|Granatenexplosionsschaden (HP)|Daño de explosión de granada (PV)
grenade_explosion_radius|Moltiplicatore esplosione granata|Multiplicateur d'explosion des grenades|Granatenexplosionsmultiplikator|Multiplicador de explosión de granada
grenade_explosion_color|Colore esplosione granata|Couleur d'explosion des grenades|Granatenexplosionsfarbe|Color de explosión de granada
grenade_color|Colore modello e bagliore granata|Couleur du modèle et de la lueur des grenades|Granatenmodell- und Leuchtfarbe|Color del modelo y brillo de granada
continuous_missiles_enabled|Missili continui sperimentali|Missiles continus expérimentaux|Experimentelle Dauerraketen|Misiles continuos experimentales
shotgun_enabled|Fucile a pompa sperimentale|Fusil à pompe expérimental|Experimentelle Schrotflinte|Escopeta experimental
shotgun_damage_hp|Danno per proiettile (HP)|Dégâts par plomb (PV)|Schaden pro Schrotprojektil (HP)|Daño por perdigón (PV)
shotgun_speed|Velocità proiettili (unità/s)|Vitesse des plombs (unités/s)|Schrotgeschwindigkeit (Einheiten/s)|Velocidad de perdigones (unidades/s)
shotgun_range|Distanza massima proiettili (unità)|Portée des plombs (unités)|Schrotreichweite (Einheiten)|Alcance de perdigones (unidades)
shotgun_size_percent|Dimensione proiettili (%)|Taille des plombs (%)|Schrotgröße (%)|Tamaño de perdigones (%)
shotgun_horizontal_spread_degrees|Rosa orizzontale (gradi)|Dispersion horizontale (degrés)|Horizontale Streuung (Grad)|Dispersión horizontal (grados)
shotgun_vertical_spread_degrees|Rosa verticale (gradi)|Dispersion verticale (degrés)|Vertikale Streuung (Grad)|Dispersión vertical (grados)
shotgun_projectile_color|Colore proiettili fucile|Couleur des plombs|Schrotprojektilfarbe|Color de perdigones
shotgun_trail_color|Colore scie fucile|Couleur des traînées des plombs|Schrotschweiffarbe|Color de estelas de perdigones
shotgun_impact_color|Colore impatto fucile|Couleur d'impact des plombs|Schrotaufprallfarbe|Color de impacto de perdigones
flame_mine_enabled|Mina di fuoco sperimentale|Mine enflammée expérimentale|Experimentelle Flammenmine|Mina de fuego experimental
mine_flame_range|Lunghezza fiamme mina (unità)|Longueur des flammes de mine (unités)|Minenflammenlänge (Einheiten)|Longitud de llamas de mina (unidades)
mine_flame_rotation_degrees_per_second|Rotazione fiamme mina (gradi/s)|Rotation des flammes de mine (degrés/s)|Minenflammenrotation (Grad/s)|Rotación de llamas de mina (grados/s)
mine_flame_damage_per_second|Danno fiamme mina (HP/s)|Dégâts des flammes de mine (PV/s)|Minenflammenschaden (HP/s)|Daño de llamas de mina (PV/s)
flame_mine_size_percent|Dimensione mina di fuoco (%)|Taille de la mine enflammée (%)|Flammenminengröße (%)|Tamaño de mina de fuego (%)
sand_mouse_enabled|Topo della sabbia sperimentale|Sandmouse expérimental|Experimentelle Sandmaus|Ratón de arena experimental
sand_mouse_mode|Modalità compagno|Mode du compagnon|Begleitermodus|Modo del acompañante
sand_mouse_lifetime_seconds|Durata compagno (secondi)|Durée du compagnon (secondes)|Begleiterdauer (Sekunden)|Duración del acompañante (segundos)
sand_mouse_max_shots|Colpi massimi compagno|Nombre maximal de tirs du compagnon|Maximale Begleiterschüsse|Máximo de disparos del acompañante
sand_mouse_fire_interval_ticks|Intervallo fuoco compagno|Intervalle de tir du compagnon|Begleiterfeuerintervall|Intervalo de disparo del acompañante
sand_mouse_shot_range|Distanza massima colpi compagno|Portée des tirs du compagnon|Begleiterschussreichweite|Alcance de disparos del acompañante
sand_mouse_shot_damage_hp|Danno colpi compagno (HP)|Dégâts des tirs du compagnon (PV)|Begleiterschussschaden (HP)|Daño de disparos del acompañante (PV)
sand_mouse_projectile_color|Colore proiettili compagno|Couleur des projectiles du compagnon|Begleiterprojektilfarbe|Color de proyectiles del acompañante
secondary_beam_damage_percent|Danno sovrapposizione secondo raggio (%)|Dégâts de chevauchement du second rayon (%)|Überlappungsschaden des zweiten Strahls (%)|Daño superpuesto del segundo rayo (%)
throw_speed|Velocità lancio Onnichiave (unità/s)|Vitesse de lancer de la clé (unités/s)|Schlüsselwurfgeschwindigkeit (Einheiten/s)|Velocidad de lanzamiento de llave (unidades/s)
show_hud|Mostra nome e XP di quest'arma nell'HUD in alto a sinistra|Afficher le nom et l'XP de cette arme en haut à gauche|Name und XP dieser Waffe oben links anzeigen|Mostrar nombre y XP de esta arma arriba a la izquierda
crate_breaker_enabled|Spacca casse|Brise-caisses|Kistenbrecher|Rompecajas
crate_breaker_radius|Raggio spacca casse (unità)|Rayon du brise-caisses (unités)|Kistenbrecherradius (Einheiten)|Radio de rompecajas (unidades)
moving_attacks_enabled|Colpi in movimento con telecamera bloccata|Attaques en mouvement avec caméra verrouillée|Angriffe in Bewegung bei gesperrter Kamera|Ataques en movimiento con cámara bloqueada
bolt_attraction_bonus|Distanza aggiuntiva attrazione bolt (unità)|Portée supplémentaire d'attraction des boulons (unités)|Zusätzliche Bolt-Anziehungsreichweite (Einheiten)|Alcance adicional de atracción de guitones (unidades)
''')

def label(key):
    from schema import title, XP
    if key in XP:
        n=XP.index(key)+1
        return tr('XP from V{grade} to V{next_grade}',grade=n,next_grade=n+1)
    if key.startswith('ammo_unlock_') and key.endswith('_bolts'):
        return tr('{weapon} unlock (collected bolts)',weapon=weapon_name(key[12:-6]))
    if key.startswith('ammo_drop_chance_'):
        return tr('{weapon} ammo chance (%)',weapon=weapon_name(key[17:]))
    return title(key) if _language=='en' else FIELD_LABELS[key][_language]

HELP_TEXT = rows('''
per_grade|Valore indipendente per questo grado dell'arma.|Valeur indépendante pour ce grade de l'arme.|Unabhängiger Wert für diese Waffenstufe.|Valor independiente para este grado del arma.
color|Colore RGB: RRGGBB oppure #RRGGBB. default conserva il colore originale. Usa il campione per scegliere un colore.|Couleur RVB : RRGGBB ou #RRGGBB. default conserve la couleur d'origine. Utilisez l'échantillon pour choisir une couleur.|RGB-Farbe: RRGGBB oder #RRGGBB. default behält die Originalfarbe bei. Farbfeld zur Auswahl anklicken.|Color RGB: RRGGBB o #RRGGBB. default conserva el color original. Usa la muestra para elegir un color.
ticks|Tick di gioco: 60 tick = 1 secondo, indipendentemente dagli FPS.|Ticks du jeu : 60 ticks = 1 seconde, indépendamment des FPS.|Spiel-Ticks: 60 Ticks = 1 Sekunde, unabhängig von den FPS.|Ticks del juego: 60 ticks = 1 segundo, independientemente de los FPS.
multiplier|Moltiplicatore del valore di riferimento: 1 = normale, 0.5 = metà, 2 = doppio.|Multiplicateur de référence : 1 = normal, 0.5 = moitié, 2 = double.|Referenzmultiplikator: 1 = normal, 0.5 = halb, 2 = doppelt.|Multiplicador de referencia: 1 = normal, 0.5 = mitad, 2 = doble.
xp|XP richiesti per questo passaggio di grado, non cumulativi. Non modifica gli XP già salvati.|XP requis pour cette transition, non cumulés. Ne modifie pas l'XP déjà sauvegardée.|Erforderliche XP für diesen Stufenwechsel, nicht kumulativ. Gespeicherte XP bleiben unverändert.|XP necesarios para esta transición, no acumulativos. No cambia los XP ya guardados.
boolean|Off = disattivato. On = attivato. L'INI conserva i valori false/true per compatibilità.|Off = désactivé. On = activé. L'INI conserve false/true pour compatibilité.|Off = aus. On = ein. Die INI behält false/true zur Kompatibilität.|Off = desactivado. On = activado. El INI conserva false/true por compatibilidad.
max_ammo|Capacità munizioni di questo grado. Caricare una configurazione non regala munizioni.|Capacité de munitions de ce grade. Charger une configuration ne donne pas de munitions.|Munitionskapazität dieser Stufe. Das Laden einer Konfiguration gibt keine Munition.|Capacidad de munición de este grado. Cargar una configuración no otorga munición.
experimental_max_ammo|Sostituisce la capacità standard quando una modalità sperimentale dell'arma è attiva. Non regala munizioni. Per il Cannone a risucchio indica gli oggetti conservabili (0-10, minimo effettivo 1). I vecchi profili ereditano il valore standard del grado.|Remplace la capacité standard lorsqu'un mode expérimental de l'arme est actif. Ne donne pas de munitions. Pour l'Aspiro-Canon : projectiles stockés (0-10, minimum effectif 1). Les anciens profils reprennent la capacité standard du grade.|Ersetzt die Standardkapazität bei aktivem experimentellem Waffenmodus. Gibt keine Munition. Bei der Saug-Kanone: gespeicherte Projektile (0-10, effektives Minimum 1). Alte Profile übernehmen die Standardkapazität der Stufe.|Sustituye la capacidad estándar al activar un modo experimental del arma. No otorga munición. Para el Cañón de succión: proyectiles almacenados (0-10, mínimo efectivo 1). Los perfiles antiguos heredan la capacidad estándar del grado.
damage_hp|HP tolti da ciascun proiettile o esplosione. Ogni istanza colpisce un nemico una volta; un'altra può colpirlo subito.|PV retirés par chaque projectile ou explosion. Chaque instance touche un ennemi une fois ; une autre peut le toucher immédiatement.|HP-Schaden je Projektil oder Explosion. Jede Instanz trifft einen Gegner einmal; die nächste darf sofort treffen.|PV quitados por proyectil o explosión. Cada instancia golpea una vez a cada enemigo; otra puede golpear inmediatamente.
damage_per_second|HP al secondo di contatto effettivo. Le particelle non moltiplicano il danno; l'intervallo di registrazione raggruppa il danno e gli XP.|PV par seconde de contact réel. Les particules ne multiplient pas les dégâts ; l'intervalle d'enregistrement regroupe dégâts et XP.|HP pro Sekunde tatsächlichen Kontakts. Partikel vervielfachen den Schaden nicht; das Erfassungsintervall bündelt Schaden und XP.|PV por segundo de contacto real. Las partículas no multiplican el daño; el intervalo de registro agrupa daño y XP.
taunter_dps|Danno al secondo dell'aura offensiva entro il raggio danni, indipendente da attrazione e stordimento.|Dégâts par seconde de l'aura offensive dans la portée des dégâts, indépendants de l'attraction et de l'étourdissement.|Schaden pro Sekunde der offensiven Aura innerhalb der Schadensreichweite, unabhängig von Anlocken und Betäubung.|Daño por segundo del aura ofensiva dentro del alcance de daño, independiente de atracción y aturdimiento.
damage_interval_ticks|Registra danni e XP ogni N tick di contatto effettivo con ciascun nemico (1-600). Non aumenta gli HP/s. Il danno parziale attende il prossimo impulso e si azzera quando il bersaglio scompare o si carica una partita.|Enregistre dégâts et XP tous les N ticks de contact réel par ennemi (1-600). N'augmente pas les PV/s. Les dégâts partiels attendent l'impulsion suivante et se réinitialisent à la disparition de la cible ou au chargement.|Erfasst Schaden und XP alle N tatsächlichen Kontakt-Ticks je Gegner (1-600). Erhöht HP/s nicht. Teilschaden wartet auf den nächsten Impuls und wird beim Entfernen des Ziels oder Laden zurückgesetzt.|Registra daño y XP cada N ticks de contacto real por enemigo (1-600). No aumenta los PV/s. El daño parcial espera al siguiente pulso y se reinicia al desaparecer el objetivo o cargar partida.
ammo_interval_ticks|Consuma una munizione ogni N tick di fuoco. Indipendente dal danno: valori maggiori consumano più lentamente.|Consomme une munition tous les N ticks de tir. Indépendant des dégâts : une valeur élevée réduit la consommation.|Verbraucht eine Munition alle N Feuer-Ticks. Unabhängig vom Schaden: höhere Werte verbrauchen langsamer.|Consume una unidad cada N ticks de disparo. Independiente del daño: valores mayores consumen más despacio.
gold_texture|Applica l'aspetto d'oro al modello 3D, senza attivare i bonus del comportamento d'oro.|Applique l'apparence en or au modèle 3D sans activer les bonus du comportement en or.|Gold-Aussehen für das 3D-Modell, ohne die Gold-Spielmechanik zu aktivieren.|Aplica el aspecto de oro al modelo 3D sin activar las bonificaciones del comportamiento de oro.
gold_effects|Attiva le meccaniche e i bonus originali dell'arma d'oro, indipendentemente dalla texture.|Active les mécanismes et bonus d'origine de l'arme en or, indépendamment de sa texture.|Aktiviert die ursprüngliche Gold-Spielmechanik und Boni unabhängig von der Textur.|Activa las mecánicas y bonificaciones originales de oro, independientemente de la textura.
auto_lock_range|default segue la distanza utile del lancio. Un numero può limitare l'aggancio ma non estendere il lancio.|default suit la portée utile du lancer. Un nombre peut limiter le verrouillage sans augmenter la portée.|default folgt der nutzbaren Wurfreichweite. Eine Zahl begrenzt die Zielerfassung, verlängert den Wurf aber nicht.|default sigue el alcance útil del lanzamiento. Un número puede limitar la fijación sin ampliar el alcance.
throw_range|Moltiplicatore di lancio del Guanto Bomba. Per l'Onnichiave è la distanza assoluta del lancio in unità, indipendente dalla velocità.|Multiplicateur du lancer du Gant à Bombes. Pour la clé : distance absolue du lancer en unités, indépendante de la vitesse.|Wurfmultiplikator des Bomben-Handschuhs. Beim Schlüssel: absolute Wurfweite in Einheiten, unabhängig von der Geschwindigkeit.|Multiplicador del Guante bombas. Para la llave: distancia absoluta de lanzamiento en unidades, independiente de la velocidad.
throw_speed|Velocità del lancio dell'Onnichiave, acquisita al lancio. Indipendente dalla distanza; il ritorno mantiene il comportamento originale.|Vitesse du lancer de la clé, capturée au lancement. Indépendante de la portée ; le retour conserve le comportement d'origine.|Schlüsselwurfgeschwindigkeit, beim Wurf erfasst. Unabhängig von der Reichweite; Rückflug bleibt ursprünglich.|Velocidad de lanzamiento de la llave, capturada al lanzar. Independiente del alcance; el regreso conserva el comportamiento original.
xp_per_damage|XP per HP realmente tolto a nemici idonei. Esempio: 0.12 assegna 12 XP per 100 HP. Il danno oltre la vita residua non assegna XP.|XP par PV réellement retiré aux ennemis éligibles. Exemple : 0.12 donne 12 XP pour 100 PV. Les dégâts excédentaires ne donnent pas d'XP.|XP pro tatsächlich entferntem HP bei geeigneten Gegnern. 0.12 gibt z. B. 12 XP für 100 HP. Überzähliger Schaden gibt keine XP.|XP por PV realmente quitado a enemigos válidos. Ejemplo: 0.12 da 12 XP por 100 PV. El exceso de daño no otorga XP.
show_hud|Solo nome, vignetta e barra XP di quest'arma in alto a sinistra. Progressione, barra nel menu pausa e notifiche restano attive.|Seulement le nom, le fond et la barre d'XP de cette arme en haut à gauche. Progression, barre du menu pause et notifications restent actives.|Nur Name, Hintergrund und XP-Leiste dieser Waffe oben links. Fortschritt, Pausemenü-Leiste und Meldungen bleiben aktiv.|Solo el nombre, fondo y barra de XP de esta arma arriba a la izquierda. Progresión, barra de pausa y avisos siguen activos.
crate_breaker_enabled|Il colpo dall'alto crea un'onda invisibile che rompe solo casse. I colpi normali non la generano e non aggiunge danno radiale ai nemici.|Le coup au sol crée une onde invisible brisant uniquement les caisses. Les attaques normales ne la déclenchent pas ; aucun dégât radial supplémentaire aux ennemis.|Der Bodenschlag erzeugt eine unsichtbare Welle nur gegen Kisten. Normale Schläge lösen sie nicht aus; kein zusätzlicher Flächenschaden an Gegnern.|El golpe desde arriba crea una onda invisible que solo rompe cajas. Los golpes normales no la activan; no añade daño de área a enemigos.
crate_breaker_radius|Distanza dall'impatto del colpo dall'alto. Solo casse distruttibili originali; contenuti ed esplosioni delle casse restano normali.|Distance depuis l'impact du coup au sol. Seulement les caisses destructibles d'origine ; contenu et explosions restent normaux.|Entfernung vom Bodenschlag. Nur ursprüngliche zerstörbare Kisten; Inhalt und Kistenexplosionen bleiben normal.|Distancia desde el impacto del golpe al suelo. Solo cajas destructibles originales; contenido y explosiones permanecen normales.
moving_attacks_enabled|Con telecamera bloccata permette movimento con la levetta durante le combo a terra. Off mantiene gli affondi originali fissi.|Avec caméra verrouillée, autorise le déplacement au stick pendant les combos au sol. Off conserve les fentes fixes d'origine.|Bei gesperrter Kamera Stick-Bewegung während Bodencombos zulassen. Off behält die festen ursprünglichen Ausfallschritte.|Con cámara bloqueada permite moverse con el stick durante combos terrestres. Off conserva los avances fijos originales.
bolt_attraction_bonus|Bonus passivo del grado corrente anche con un'altra arma in mano. Si somma al raggio originale e al Bolt Grabber. Non cambia valore dei bolt o raccolta munizioni.|Bonus passif du grade actuel, même avec une autre arme. S'ajoute au rayon d'origine et à l'Attrape-Boulons. Ne change ni valeur des boulons ni collecte de munitions.|Passiver Bonus der aktuellen Stufe auch bei anderer gehaltener Waffe. Addiert sich zum ursprünglichen Radius und Bolt-Greifer. Ändert weder Bolt-Wert noch Munitionsaufnahme.|Bonificación pasiva del grado actual, incluso con otra arma equipada. Se suma al radio original y al imán de guitones. No cambia el valor de guitones ni la recogida de munición.
experimental_ammo_unlock_mode|Le nuove partite iniziano senza bombe. Le armi si sbloccano con munizioni e si ribloccano quando finiscono; ogni sblocco aggiunge l'icona rapida. Negozi disattivati. Le casse danno 1-4 caricatori, migliorati dai bolt d'oro. Le armi senza munizioni usano le soglie bolt. Stato per slot; disattivare ripristina il possesso originale.|Les nouvelles parties commencent sans bombes. Les armes se débloquent avec les munitions et se rebloquent à zéro ; chaque déblocage ajoute l'icône rapide. Magasins désactivés. Les caisses donnent 1-4 packs, améliorés par les boulons en or. Les armes sans munitions utilisent les seuils de boulons. État par sauvegarde ; désactiver restaure la possession d'origine.|Neue Spiele starten ohne Bomben. Munition schaltet Waffen frei; bei null werden sie gesperrt. Jede Freischaltung setzt das Schnellwahlsymbol. Keine Händler. Kisten geben 1-4 Pakete, verbessert durch Gold-Bolts. Munitionsfreie Waffen nutzen Bolt-Schwellen. Zustand je Spielstand; Deaktivieren stellt ursprünglichen Besitz wieder her.|Las partidas nuevas empiezan sin bombas. La munición desbloquea armas y al agotarse las bloquea; cada desbloqueo añade el icono rápido. Tiendas desactivadas. Las cajas dan 1-4 cargadores, mejorados con guitones de oro. Armas sin munición usan umbrales de guitones. Estado por partida; desactivar restaura la posesión original.
unlock_threshold|Bolt normali raccolti necessari per lo sblocco permanente. Non è un acquisto: non spende bolt. 0 sblocca subito. Solo in Ammo-Survival Mode.|Boulons ordinaires collectés nécessaires au déblocage permanent. Ce n'est pas un achat : aucun boulon dépensé. 0 débloque immédiatement. Seulement dans Ammo-Survival Mode.|Gesammelte normale Bolts für dauerhafte Freischaltung. Kein Kauf: verbraucht keine Bolts. 0 schaltet sofort frei. Nur im Ammo-Survival Mode.|Guitones normales recogidos para el desbloqueo permanente. No es una compra: no gasta guitones. 0 desbloquea al instante. Solo en Ammo-Survival Mode.
ammo_chance|Peso per ciascun caricatore indipendente. Tutti i valori vengono normalizzati al 100%. 0 esclude il tipo; almeno uno deve essere positivo. Solo in Ammo-Survival Mode.|Poids par pack indépendant. Toutes les valeurs sont normalisées à 100 %. 0 exclut ce type ; au moins une valeur doit être positive. Seulement dans Ammo-Survival Mode.|Gewicht je unabhängigem Paket. Alle Werte werden auf 100 % normiert. 0 schließt den Typ aus; mindestens einer muss positiv sein. Nur im Ammo-Survival Mode.|Peso por cargador independiente. Los valores se normalizan al 100 %. 0 excluye el tipo; al menos uno debe ser positivo. Solo en Ammo-Survival Mode.
dual_beam_enabled|Due raggi elettrici originali indipendenti, anche senza bersagli. Il secondo preferisce un altro nemico; sullo stesso usa la percentuale di sovrapposizione. Una sola spesa munizioni e un solo suono.|Deux rayons électriques d'origine indépendants, même sans cible. Le second préfère un autre ennemi ; sur le même, il utilise le pourcentage de chevauchement. Une seule consommation et un seul son.|Zwei unabhängige ursprüngliche Elektrostrahlen, auch ohne Ziel. Der zweite bevorzugt einen anderen Gegner; am selben gilt der Überlappungsprozentsatz. Einmal Munition, ein Ton.|Dos rayos eléctricos originales independientes, incluso sin objetivos. El segundo prefiere otro enemigo; sobre el mismo usa el porcentaje de superposición. Un solo consumo y un solo sonido.
secondary_beam_damage_percent|Percentuale del danno base del secondo raggio sullo stesso nemico: 25 = 25% degli HP/s. Su un nemico diverso infligge il danno base completo.|Pourcentage des dégâts de base du second rayon sur le même ennemi : 25 = 25 % des PV/s. Sur un ennemi différent, dégâts complets.|Basisschadensanteil des zweiten Strahls am selben Gegner: 25 = 25 % der HP/s. An einem anderen Gegner voller Basisschaden.|Porcentaje del daño base del segundo rayo sobre el mismo enemigo: 25 = 25 % de PV/s. En otro enemigo causa el daño base completo.
beam_size_percent|Larghezza complessiva del raggio: 100 è originale. Non modifica danni al secondo o distanza.|Largeur globale du rayon : 100 est l'original. Ne change ni dégâts par seconde ni portée.|Gesamte Strahlbreite: 100 ist ursprünglich. Ändert weder Schaden pro Sekunde noch Reichweite.|Anchura general del rayo: 100 es original. No cambia daño por segundo ni alcance.
beam_range_multiplier|Scala distanza visiva e danni del raggio. Per Tesla moltiplica anche la distanza bersaglio.|Multiplie la portée visuelle et les dégâts à distance du rayon. Pour Tesla, multiplie aussi la portée des cibles.|Skaliert sichtbare und schädigende Strahlreichweite. Bei Tesla zusätzlich Multiplikator der Zielreichweite.|Escala el alcance visual y dañino del rayo. Para Tesla también multiplica el alcance del objetivo.
cluster_bombs_enabled|Rilascia mini bombe visibili prima dell'esplosione su una superficie. Le mini bombe non generano altre mini bombe.|Libère des mini-bombes visibles avant l'explosion sur une surface. Elles ne créent pas d'autres mini-bombes.|Setzt sichtbare Minibomben vor der Oberflächenexplosion frei. Minibomben erzeugen keine weiteren Minibomben.|Libera minibombas visibles antes de la explosión contra una superficie. No generan más minibombas.
cluster_jump_speed|Velocità iniziale verso l'alto in unità/s. Gravità originale; le esplosioni al contatto si abilitano all'apice. Valore predefinito 4.2.|Vitesse initiale ascendante en unités/s. Gravité d'origine ; l'explosion au contact s'arme au sommet. Valeur par défaut 4.2.|Anfängliche Aufwärtsgeschwindigkeit in Einheiten/s. Ursprüngliche Schwerkraft; Kontaktzündung wird am Scheitelpunkt scharf. Standard 4.2.|Velocidad inicial ascendente en unidades/s. Gravedad original; la detonación por contacto se activa en el ápice. Predeterminado 4.2.
cluster_damage_percent|Danno percentuale rispetto al grado della bomba principale, acquisito alla creazione. Predefinito 10%.|Pourcentage des dégâts du grade de la bombe principale, capturé à la création. Par défaut 10 %.|Prozentualer Schaden der Hauptbombenstufe, bei Erzeugung erfasst. Standard 10 %.|Porcentaje de daño del grado de la bomba principal, capturado al crearla. Predeterminado 10 %.
cluster_explosion_size_percent|Effetto visivo e raggio danni in percentuale dell'esplosione principale. Predefinito 15%.|Effet visuel et rayon des dégâts en pourcentage de l'explosion principale. Par défaut 15 %.|Effekt und Schadensradius in Prozent der Hauptexplosion. Standard 15 %.|Efecto visual y radio de daño como porcentaje de la explosión principal. Predeterminado 15 %.
cluster_bomb_size_percent|Dimensione del modello rispetto alla bomba principale. Predefinito 40%.|Taille du modèle par rapport à la bombe principale. Par défaut 40 %.|Modellgröße relativ zur Hauptbombe. Standard 40 %.|Tamaño del modelo respecto a la bomba principal. Predeterminado 40 %.
triple_flame_enabled|Due getti aggiuntivi condividono bocca, consumo munizioni e danni al secondo. La sovrapposizione non moltiplica il danno su un nemico.|Deux jets supplémentaires partagent bouche, consommation et dégâts par seconde. Le chevauchement ne multiplie pas les dégâts sur un ennemi.|Zwei zusätzliche Strahlen teilen Mündung, Munitionsverbrauch und Schaden pro Sekunde. Überlappung vervielfacht Schaden am selben Gegner nicht.|Dos chorros adicionales comparten boca, consumo y daño por segundo. La superposición no multiplica el daño al mismo enemigo.
crate_suction_enabled|Usa le casse consentite come proiettili, conservando modello e contenuto fino all'impatto. Acciaio: danno doppio. Le esplosioni TNT assegnano XP al Cannone a risucchio per gli HP tolti.|Utilise les caisses autorisées comme projectiles, conservant modèle et contenu jusqu'à l'impact. Acier : dégâts doubles. Les explosions TNT donnent de l'XP à l'Aspiro-Canon selon les PV retirés.|Erlaubte Kisten als Projektile mit Modell und Inhalt bis zum Aufprall. Stahl verursacht doppelten Schaden. TNT-Schaden vergibt Saug-Kanonen-XP nach entfernten HP.|Usa las cajas permitidas como proyectiles, conservando modelo y contenido hasta impactar. Acero: daño doble. Las explosiones TNT otorgan XP al Cañón de succión por PV quitados.
suck_crate|La cassa mantiene contenuto e ricompense originali fino alla distruzione dopo il lancio. Le casse normali usano il danno del grado; l'acciaio il doppio; quelle esplosive detonano e assegnano XP.|La caisse garde son contenu et ses récompenses jusqu'à sa destruction après le tir. Dégâts du grade pour les caisses normales ; doubles pour l'acier ; les explosives détonent et donnent de l'XP.|Die Kiste behält Inhalt und Belohnung bis zur Zerstörung nach dem Schuss. Normale Kisten nutzen Stufenschaden, Stahl den doppelten; explosive detonieren und geben XP.|La caja conserva contenido y recompensas hasta destruirse tras el disparo. Las normales usan daño del grado; acero el doble; explosivas detonan y otorgan XP.
continuous_missiles_enabled|Spara un missile per intervallo mentre tieni premuto fuoco, consumando una munizione. Sostituisce la raffica originale; incompatibile con il fucile a pompa.|Tire un missile par intervalle en maintenant le tir, pour une munition. Remplace la salve d'origine ; incompatible avec le fusil à pompe.|Eine Rakete je Intervall bei gehaltenem Feuer, eine Munition je Rakete. Ersetzt die ursprüngliche Salve; nicht mit Schrotmodus kombinierbar.|Dispara un misil por intervalo al mantener fuego, consumiendo una unidad. Sustituye la ráfaga original; incompatible con escopeta.
shotgun_enabled|Spara 35 proiettili indipendenti in una griglia di sette colonne e cinque righe. Una munizione per raffica. Sostituisce i missili; incompatibile con missili continui.|Tire 35 projectiles indépendants dans une grille de sept colonnes et cinq lignes. Une munition par salve. Remplace les missiles ; incompatible avec les missiles continus.|Feuert 35 unabhängige Projektile in sieben Spalten und fünf Reihen. Eine Munition je Salve. Ersetzt Raketen; nicht mit Dauerraketen kombinierbar.|Dispara 35 proyectiles independientes en siete columnas y cinco filas. Una unidad por ráfaga. Sustituye misiles; incompatible con misiles continuos.
shotgun_damage_hp|Danno di ciascuno dei 35 proiettili, acquisito al lancio. Ogni proiettile colpisce un nemico una volta; il danno non viene diviso per il numero di proiettili.|Dégâts de chacun des 35 projectiles, capturés au tir. Chaque projectile touche une fois par ennemi ; les dégâts ne sont pas divisés par leur nombre.|Schaden jedes der 35 Projektile, beim Schuss erfasst. Je Projektil ein Treffer pro Gegner; Schaden wird nicht durch die Projektilzahl geteilt.|Daño de cada uno de los 35 proyectiles, capturado al disparar. Cada proyectil golpea una vez por enemigo; no se divide el daño entre proyectiles.
shotgun_spread|Angolo massimo delle file o colonne esterne; quelle intermedie sono proporzionali. 0 mantiene i proiettili paralleli. Entrambi gli assi a 0 mantengono comunque una griglia di 35 origini distinte.|Angle maximal des rangées ou colonnes extérieures ; intermédiaires proportionnelles. 0 garde les projectiles parallèles. Les deux axes à 0 conservent une grille de 35 origines distinctes.|Maximalwinkel der äußeren Reihen oder Spalten; innere proportional. 0 hält Projektile parallel. Beide Achsen bei 0 behalten 35 getrennte Gitterursprünge.|Ángulo máximo de filas o columnas exteriores; intermedias proporcionales. 0 mantiene proyectiles paralelos. Ambos ejes en 0 conservan 35 orígenes distintos en cuadrícula.
missile_max_range|Distanza percorsa dalla bocca in unità. 0 conserva il limite originale. Al raggiungimento il missile esplode.|Distance parcourue depuis la bouche en unités. 0 conserve la limite d'origine. Le missile explose à la limite.|Reisedistanz ab Mündung in Einheiten. 0 behält die ursprüngliche Grenze. Bei Erreichen explodiert die Rakete.|Distancia recorrida desde la boca en unidades. 0 conserva el límite original. El misil explota al alcanzarlo.
explosion_visual_size_multiplier|Modifica solo l'effetto visivo; il raggio dei danni è separato.|Modifie uniquement l'effet visuel ; le rayon des dégâts est séparé.|Ändert nur den sichtbaren Effekt; der Schadensradius ist getrennt.|Solo cambia el efecto visual; el radio de daño es independiente.
explosion_damage_radius|Raggio danni in unità. Per R.Y.N.O. 0 conserva il solo impatto diretto originale. La dimensione visiva è separata.|Rayon des dégâts en unités. Pour le T.E.L.T., 0 conserve l'impact direct d'origine. Taille visuelle séparée.|Schadensradius in Einheiten. Bei R.I.D.A. behält 0 den ursprünglichen direkten Einschlag bei. Sichtbare Größe getrennt.|Radio de daño en unidades. Para T.A.U.N., 0 conserva el impacto directo original. Tamaño visual independiente.
flame_mine_enabled|Dopo l'atterraggio sostituisce attivazione, inseguimento ed esplosione con quattro bracci di fuoco rotanti. Ogni mina ha suono spaziale e XP indipendenti.|Après l'atterrissage, remplace activation, poursuite et explosion par quatre bras de feu tournants. Chaque mine a un son spatial et de l'XP indépendants.|Nach Landung ersetzen vier rotierende Flammenarme Aktivierung, Verfolgung und Explosion. Jede Mine hat unabhängigen räumlichen Ton und XP.|Tras aterrizar sustituye activación, persecución y explosión por cuatro brazos de fuego giratorios. Cada mina tiene sonido espacial y XP independientes.
mine_flame_damage_per_second|HP al secondo di contatto. I bracci della stessa mina non moltiplicano il danno sullo stesso nemico; mine diverse sono indipendenti.|PV par seconde de contact. Les bras d'une même mine ne multiplient pas les dégâts sur un ennemi ; les mines distinctes sont indépendantes.|HP pro Kontaktsekunde. Arme derselben Mine vervielfachen den Schaden am selben Gegner nicht; verschiedene Minen sind unabhängig.|PV por segundo de contacto. Los brazos de la misma mina no multiplican el daño al mismo enemigo; minas diferentes son independientes.
mine_flame_rotation_degrees_per_second|Velocità di rotazione dei quattro bracci e della mina. 0 mantiene fissa la croce.|Vitesse de rotation des quatre bras et de la mine. 0 garde la croix fixe.|Rotationsgeschwindigkeit der vier Arme und Mine. 0 hält das Kreuz fest.|Velocidad de rotación de los cuatro brazos y la mina. 0 mantiene la cruz fija.
flame_mine_size_percent|Solo mine sperimentali: 100 originale, 150 predefinito. Altezza da terra proporzionale al modello. Corpo e fiamme ruotano insieme.|Mines expérimentales seulement : 100 original, 150 par défaut. Garde au sol proportionnelle au modèle. Corps et flammes tournent ensemble.|Nur experimentelle Minen: 100 ursprünglich, 150 Standard. Bodenabstand skaliert mit Modell. Körper und Flammen rotieren gemeinsam.|Solo minas experimentales: 100 original, 150 predeterminado. Altura sobre suelo proporcional al modelo. Cuerpo y llamas giran juntos.
max_active_mines|Numero massimo di mine. Lanciarne un'altra elimina la più vecchia senza esplosione dannosa.|Nombre maximal de mines. En lancer une autre retire la plus ancienne sans explosion dommageable.|Maximale Minenanzahl. Eine weitere entfernt die älteste ohne schädigende Explosion.|Máximo de minas. Lanzar otra retira la más antigua sin explosión dañina.
mine_lifetime_seconds|Tempo dal lancio alla rimozione automatica, incluso il volo. Vale per mine normali e sperimentali.|Temps du lancer au retrait automatique, vol compris. Concerne mines normales et expérimentales.|Zeit ab Wurf bis automatischer Entfernung einschließlich Flug. Für normale und experimentelle Minen.|Tiempo desde el lanzamiento hasta la retirada automática, incluido vuelo. Para minas normales y experimentales.
sand_mouse_enabled|Evoca il compagno volante usando munizioni dello Scudo Drone; i suoi danni assegnano XP allo Scudo Drone. Uno solo attivo, comprese le tane originali; un altro tentativo non consuma munizioni.|Invoque le compagnon volant avec les munitions du Dronomatic ; ses dégâts donnent de l'XP au Dronomatic. Un seul actif, tanières d'origine comprises ; une nouvelle tentative ne consomme rien.|Ruft den fliegenden Begleiter mit Abwehr-Drohnen-Munition; sein Schaden gibt der Waffe XP. Nur einer aktiv, einschließlich ursprünglicher Baue; weitere Versuche verbrauchen nichts.|Invoca al acompañante volador con munición del Guante defensor; sus daños dan XP a esa arma. Solo uno activo, incluidas madrigueras originales; otro intento no consume munición.
sand_mouse_mode|1: solo compagno. 2: compagno e droni. 3: compagno con Sonic Summoner indossato, altrimenti droni. Un compagno attivo impedisce altre evocazioni.|1 : compagnon seul. 2 : compagnon et drones. 3 : compagnon avec l'Appâte-Sandmouse équipé, drones sinon. Un compagnon actif bloque d'autres invocations.|1: nur Begleiter. 2: Begleiter und Drohnen. 3: Begleiter bei getragenem Schall-Summoner, sonst Drohnen. Ein aktiver Begleiter blockiert weitere Beschwörungen.|1: solo acompañante. 2: acompañante y drones. 3: acompañante con Invocador sónico equipado, drones en otro caso. Un acompañante activo bloquea otras invocaciones.
sand_mouse_lifetime_seconds|Durata indipendente dal possesso o equipaggiamento del Sonic Summoner. Cresce dall'1% alla dimensione originale in mezzo secondo.|Durée indépendante de la possession ou de l'équipement de l'Appâte-Sandmouse. Grandit de 1 % à la taille d'origine en une demi-seconde.|Dauer unabhängig von Besitz oder Tragen des Schall-Summoners. Wächst in einer halben Sekunde von 1 % auf Originalgröße.|Duración independiente de poseer o equipar el Invocador sónico. Crece del 1 % al tamaño original en medio segundo.
sand_mouse_max_shots|Rimuove il compagno dopo questo numero di colpi creati con successo, anche prima del termine della durata.|Retire le compagnon après ce nombre de tirs créés avec succès, même avant la fin de sa durée.|Entfernt den Begleiter nach dieser Anzahl erfolgreich erzeugter Schüsse, auch vor Ablauf seiner Dauer.|Retira al acompañante tras este número de disparos creados con éxito, incluso antes de agotar su duración.
sand_mouse_shot_damage_hp|HP tolti da ogni colpo del compagno. La perdita effettiva di vita assegna XP allo Scudo Drone.|PV retirés par chaque tir du compagnon. Les PV réellement perdus donnent de l'XP au Dronomatic.|HP je Begleiterschuss. Tatsächlich verlorene Gegner-HP vergeben XP an die Abwehr-Drohnen-Waffe.|PV quitados por disparo del acompañante. La pérdida real de vida otorga XP al Guante defensor.
sand_mouse_shot_range|Distanza massima del colpo e dell'acquisizione bersagli, in unità di gioco.|Portée maximale des tirs et d'acquisition des cibles, en unités du jeu.|Maximale Schuss- und Zielerfassungsreichweite in Spieleinheiten.|Alcance máximo del disparo y adquisición de objetivos, en unidades del juego.
grenade_launcher_enabled|Sostituisce i missili guidati con granate rotanti. Ratchet rimane libero di muoversi. Usa munizioni e XP della Visibomba, danno indipendente e mirino sul terreno.|Remplace les missiles guidés par des grenades tournantes. Ratchet reste libre de bouger. Munitions et XP du Canon Visibombe, dégâts indépendants et marqueur au sol.|Ersetzt gelenkte Raketen durch rotierende Granaten. Ratchet bleibt frei beweglich. Munition und XP des Visibombenwerfers, unabhängiger Schaden und Bodenmarkierung.|Sustituye misiles guiados por granadas giratorias. Ratchet puede moverse libremente. Usa munición y XP del Visicañón, daño independiente y marcador en el suelo.
grenade_max_range|Distanza orizzontale dalla bocca. Al limite la granata cade verticalmente; esplode solo al contatto, anche con acqua. Pozzi mortali la eliminano in silenzio. Il mirino indica solo terreno reale.|Portée horizontale depuis la bouche. À la limite, la grenade tombe verticalement ; elle explose au contact, eau comprise. Les zones mortelles la suppriment sans bruit. Le marqueur indique seulement le terrain réel.|Horizontale Entfernung ab Mündung. An der Grenze fällt die Granate senkrecht; Explosion nur bei Kontakt, auch Wasser. Todeszonen entfernen sie lautlos. Markierung nur auf echter Geometrie.|Alcance horizontal desde la boca. Al límite cae verticalmente; explota solo por contacto, incluida agua. Zonas mortales la eliminan en silencio. El marcador solo indica terreno real.
grenade_arc_height|Altezza dell'apice sopra il punto più alto della traiettoria tra partenza e arrivo, in unità. Gravità e previsione condividono l'integrazione originale.|Hauteur du sommet au-dessus de l'extrémité la plus haute, en unités. Gravité et aperçu partagent l'intégration d'origine.|Scheitelhöhe über dem höheren Endpunkt in Einheiten. Schwerkraft und Vorschau teilen die ursprüngliche Integration.|Altura del ápice sobre el extremo más alto, en unidades. Gravedad y vista previa comparten la integración original.
grenade_damage_hp|HP tolti una volta per nemico da ogni esplosione. Una granata diversa può colpire subito. Assegna XP alla Visibomba.|PV retirés une fois par ennemi et par explosion. Une autre grenade peut toucher immédiatement. Donne de l'XP au Canon Visibombe.|HP-Schaden einmal je Gegner und Explosion. Eine weitere Granate darf sofort treffen. Vergibt Visibombenwerfer-XP.|PV quitados una vez por enemigo y explosión. Otra granada puede golpear inmediatamente. Otorga XP al Visicañón.
grenade_explosion_radius|Moltiplicatore dell'esplosione originale del Guanto Bomba, effetto e danni. 1 raggiunge 2.5 unità di raggio finale; indipendente dal modello.|Multiplicateur de l'explosion d'origine du Gant à Bombes, effets et dégâts. 1 atteint un rayon final de 2.5 unités ; indépendant du modèle.|Multiplikator der ursprünglichen Bomben-Handschuh-Explosion, Effekt und Schaden. 1 erreicht 2.5 Einheiten Endradius; unabhängig vom Modell.|Multiplicador de explosión original del Guante bombas, efecto y daño. 1 alcanza 2.5 unidades de radio final; independiente del modelo.
grenade_size_percent|100 indica il modello originale del Guanto Bomba; predefinito 250%. Indipendente dal raggio d'esplosione.|100 correspond au modèle d'origine du Gant à Bombes ; 250 % par défaut. Indépendant du rayon d'explosion.|100 ist das ursprüngliche Bomben-Handschuh-Modell; Standard 250 %. Unabhängig vom Explosionsradius.|100 es el modelo original del Guante bombas; predeterminado 250 %. Independiente del radio de explosión.
grenade_fire_interval_ticks|60 tick al secondo. Una munizione Visibomba per granata creata. Modello mancante o tabella oggetti piena non consumano nulla.|60 ticks par seconde. Une munition du Canon Visibombe par grenade créée. Modèle absent ou table pleine : aucune consommation.|60 Ticks pro Sekunde. Eine Visibombenmunition je erzeugter Granate. Fehlendes Modell oder volle Objekttabelle verbrauchen nichts.|60 ticks por segundo. Una munición del Visicañón por granada creada. Modelo ausente o tabla llena no consumen nada.
drone_count|Numero massimo droni attivi (0-10). Oltre sei, l'orbita condivisa si allarga per mantenerli separati.|Nombre maximal de drones actifs (0-10). Au-delà de six, l'orbite commune s'agrandit pour les espacer.|Maximal aktive Drohnen (0-10). Über sechs erweitert sich die gemeinsame Umlaufbahn für Abstand.|Máximo de drones activos (0-10). Por encima de seis la órbita compartida se amplía para separarlos.
held_lunge_duration_percent|100 è originale. Valori maggiori estendono la spinta e la finestra del colpo mentre tieni premuto fuoco, una volta per nemico per pugno.|100 est l'original. Les valeurs supérieures prolongent propulsion et fenêtre de frappe en maintenant le tir, une fois par ennemi et par coup.|100 ist ursprünglich. Höhere Werte verlängern Vortrieb und Trefferfenster bei gehaltenem Feuer, einmal je Gegner und Schlag.|100 es original. Valores mayores prolongan impulso y ventana de golpe al mantener fuego, una vez por enemigo y puñetazo.
name_opacity_percent|Opacità di nome, grado e sfondo: 100 normale, 50 semitrasparente, 0 nascosto. Non modifica modello 3D o barra XP.|Opacité du nom, du grade et du fond : 100 normal, 50 semi-transparent, 0 masqué. Ne change ni modèle 3D ni barre d'XP.|Deckkraft von Name, Stufe und Hintergrund: 100 normal, 50 halbtransparent, 0 verborgen. Ändert weder 3D-Modell noch XP-Leiste.|Opacidad del nombre, grado y fondo: 100 normal, 50 semitransparente, 0 oculto. No cambia modelo 3D ni barra de XP.
xp_bar_horizontal_inset_px|Margine orizzontale del nome. La barra XP resta allineata ai bordi esterni del riquadro munizioni.|Marge horizontale du nom. La barre d'XP reste alignée sur les bords extérieurs du panneau de munitions.|Horizontaler Namensabstand. XP-Leiste bleibt an den äußeren Rändern der Munitionsanzeige ausgerichtet.|Margen horizontal del nombre. La barra de XP permanece alineada con los bordes exteriores del panel de munición.
fire_in_air|Permette al Morforaggio di sparare in aria a questo grado. Correndo può sempre sparare.|Permet au Morpho-Rayon de tirer en l'air à ce grade. Le tir en courant est toujours possible.|Erlaubt dem Wandlerstrahl das Feuern in der Luft auf dieser Stufe. Im Laufen immer möglich.|Permite al Rayo mórfico disparar en el aire en este grado. Siempre puede disparar al correr.
stun_launch_distance|Spinta aggiuntiva su stordimento riuscito, con controllo collisioni. 0 mantiene il rinculo originale.|Projection supplémentaire après étourdissement réussi, avec contrôle des collisions. 0 conserve le recul d'origine.|Zusätzlicher Stoß bei erfolgreicher Betäubung mit Kollisionsprüfung. 0 behält ursprünglichen Rückstoß.|Empuje adicional tras aturdir con éxito, con control de colisiones. 0 conserva el retroceso original.
wave_alpha|Opacità originale dell'onda: 0 invisibile, 128 piena. Valore originale 20.|Opacité de l'onde : 0 invisible, 128 complète. Valeur d'origine 20.|Wellendeckkraft: 0 unsichtbar, 128 voll. Ursprünglicher Wert 20.|Opacidad de onda: 0 invisible, 128 completa. Valor original 20.
visual_wave|Modifica solo l'onda visiva: non aumenta danno, attrazione o stordimento.|Modifie seulement l'onde visuelle : n'augmente ni dégâts, ni attraction, ni étourdissement.|Ändert nur die sichtbare Welle: erhöht weder Schaden noch Anlocken oder Betäubung.|Solo cambia la onda visual: no aumenta daño, atracción ni aturdimiento.
lure_range|Distanza di attrazione. Per il Guanto Esca default mantiene la distanza originale specifica di ogni nemico.|Portée d'attraction. Pour le Gant à Leurres, default conserve la distance d'origine de chaque ennemi.|Anlockreichweite. Beim Köder-Handschuh behält default die ursprüngliche gegnerspezifische Entfernung.|Alcance de atracción. Para el Guante señuelo default conserva la distancia original de cada enemigo.
xp_attract_fraction|Frazione degli HP residui del nemico usata per XP da attrazione. 0.333333 è circa un terzo.|Fraction des PV restants de l'ennemi utilisée pour l'XP d'attraction. 0.333333 vaut environ un tiers.|Anteil der verbleibenden Gegner-HP für Anlock-XP. 0.333333 entspricht etwa einem Drittel.|Fracción de PV restantes del enemigo para XP por atracción. 0.333333 es aproximadamente un tercio.
level_up_sound|Suono di notifica oppure Off per disattivare solo il suono.|Son de notification ou Off pour désactiver uniquement le son.|Benachrichtigungston oder Off, um nur den Ton abzuschalten.|Sonido de notificación u Off para desactivar solo el sonido.
''')



HELP_TEXT.update(rows('''
wrench_hit_radius_multiplier|Moltiplicatore delle collisioni di tutti i tre colpi della combo, del colpo in aria, del lancio in andata e ritorno e dell'impatto dall'alto. 1 è originale, 2 raddoppia portata e spessore. Non modifica modello visivo, distanza del lancio o raggio Spacca casse. Acquisito all'inizio di ogni attacco.|Multiplicateur des collisions des trois coups du combo, du coup aérien, du lancer aller-retour et de l'impact au sol. 1 est l'original, 2 double portée et épaisseur. Ne change ni modèle, ni distance du lancer, ni rayon du brise-caisses. Capturé au début de chaque attaque.|Kollisionsmultiplikator für alle drei Comboschläge, Lufttreffer, Hin- und Rückwurf und Bodeneinschlag. 1 ist ursprünglich, 2 verdoppelt Reichweite und Dicke. Ändert weder Modell, Wurfweite noch Kistenbrecherradius. Bei Angriffsbeginn erfasst.|Multiplicador de colisiones de los tres golpes del combo, golpe aéreo, lanzamiento de ida y vuelta e impacto al suelo. 1 es original, 2 duplica alcance y grosor. No cambia modelo, distancia de lanzamiento ni radio de rompecajas. Capturado al iniciar cada ataque.
wrench_knockback_multiplier|Moltiplicatore della spinta orizzontale dei nemici per tutti i colpi dell'Onnichiave. 1 originale, 0 nessuna spinta orizzontale, 2 doppia forza. Altezza di lancio, danni, XP, animazioni e resistenza originale dei nemici restano invariati. Acquisito all'inizio dell'attacco o del lancio.|Multiplicateur de poussée horizontale des ennemis pour toutes les attaques de la clé. 1 original, 0 aucune poussée horizontale, 2 force double. Hauteur de projection, dégâts, XP, animations et résistance native inchangés. Capturé au début de l'attaque ou du lancer.|Horizontaler Stoßmultiplikator für alle Schlüsselangriffe. 1 ursprünglich, 0 kein horizontaler Stoß, 2 doppelte Stärke. Wurfhöhe, Schaden, XP, Animationen und ursprüngliche Gegnerresistenz unverändert. Bei Angriffs- oder Wurfbeginn erfasst.|Multiplicador de empuje horizontal de enemigos para todos los ataques de llave. 1 original, 0 sin empuje horizontal, 2 fuerza doble. Altura de lanzamiento, daño, XP, animaciones y resistencia original permanecen iguales. Capturado al iniciar ataque o lanzamiento.
'''))

def help_text(section,key):
    from schema import help_text as english_help, BOOLEANS, COLORS, XP
    if _language=='en': return english_help(section,key)
    address=f'[{section}] {key}' # Stable technical address for manual INI edits.
    if section.startswith('weapon.wrench.level') and key in ('hit_radius_multiplier','knockback_multiplier'):info='wrench_'+key
    elif key in COLORS: info='color'
    elif key in XP: info='xp'
    elif key.startswith('ammo_unlock_'): info='unlock_threshold'
    elif key.startswith('ammo_drop_chance_'): info='ammo_chance'
    elif key=='damage_per_second' and section.startswith('weapon.taunter.'):info='taunter_dps'
    elif key in HELP_TEXT:info=key
    elif key.startswith('suck_') and key.endswith('_crates'):info='suck_crate'
    elif key.startswith('wave_'):info='visual_wave'
    elif key.startswith('shotgun_') and key.endswith('_degrees'):info='shotgun_spread'
    else:info='per_grade'
    text=HELP_TEXT[info][_language]
    if key in BOOLEANS:text+=' '+HELP_TEXT['boolean'][_language]
    if key.endswith('_ticks'):text+=' '+HELP_TEXT['ticks'][_language]
    elif key.endswith('_multiplier') and info=='per_grade':text+=' '+HELP_TEXT['multiplier'][_language]
    return address+'\n'+label(key)+' — '+text
