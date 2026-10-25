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
    # New fields remain usable while their translations are being added.
    return title(key) if _language=='en' else FIELD_LABELS.get(key, {}).get(_language, title(key))

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
flame_mine_size_percent|Dimensione del modello sperimentale selezionato: mina, bomba o sfera del Guanto del fato. 100 è la dimensione originale del modello, 200 la raddoppia. Distanza dal terreno proporzionale; corpo e fiamme ruotano insieme.|Taille du modèle expérimental choisi : mine, bombe ou sphère du Gant Ravageur. 100 conserve sa taille native, 200 la double. Garde au sol proportionnelle ; corps et flammes tournent ensemble.|Größe jedes gewählten experimentellen Modells: Mine, Bombe oder Kugel des Inferno-Handschuhs. 100 ist die ursprüngliche Modellgröße, 200 verdoppelt sie. Bodenabstand skaliert; Körper und Flammen rotieren gemeinsam.|Tamaño del modelo experimental seleccionado: mina, bomba o esfera del Guante de destrucción. 100 conserva su tamaño original, 200 lo duplica. Altura del suelo proporcional; cuerpo y llamas giran juntos.
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
    if section.startswith('weapon.wrench.level') and key in ('hit_radius_multiplier','knockback_multiplier','trail_color'):info='wrench_'+key
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


_MODEL_NAMES = {
    'Bomb Glove bomb': ('bomb_glove', {'en':'bomb','it':'bomba','fr':'bombe','de':'Bombe','es':'bomba'}),
    'Mine Glove mine': ('mine_glove', {'en':'mine','it':'mina','fr':'mine','de':'Mine','es':'mina'}),
    'Visibomb missile': ('visibomb', {'en':'missile','it':'missile','fr':'missile','de':'Rakete','es':'misil'}),
    'Glove of Doom sphere': ('glove_of_doom', {'en':'glowing sphere','it':'sfera luminosa','fr':'sphère lumineuse','de':'Leuchtkugel','es':'esfera luminosa'}),
}
def model_name(name):
    weapon, nouns = _MODEL_NAMES[name]
    return weapon_name(weapon) + ' — ' + nouns[_language]

FIELD_LABELS.update(rows('''
grenade_model|Modello proiettile|Modèle du projectile|Projektilmodell|Modelo del proyectil
flame_mine_model|Modello mina|Modèle de mine|Minenmodell|Modelo de mina
'''))
from schema import MODEL_COLOR_GROUPS as _MODEL_COLORS
_color_labels = {
    'body': {'en':'Model color','it':'Colore modello','fr':'Couleur du modèle','de':'Modellfarbe','es':'Color del modelo'},
    'explosion': {'en':'Explosion color','it':"Colore esplosione",'fr':"Couleur de l’explosion",'de':'Explosionsfarbe','es':'Color de explosión'},
    'core': {'en':'Inner flame color','it':'Colore interno fiamma','fr':'Couleur du cœur de flamme','de':'Farbe des Flammenkerns','es':'Color del núcleo de llama'},
    'outer': {'en':'Outer flame color','it':'Colore esterno fiamma','fr':'Couleur extérieure de flamme','de':'Äußere Flammenfarbe','es':'Color exterior de llama'},
    'burn': {'en':'Persistent flame color','it':'Colore fiamma persistente','fr':'Couleur de flamme persistante','de':'Farbe der Brandflamme','es':'Color de llama persistente'},
}
for _groups in _MODEL_COLORS.values():
    for _group in _groups:
        for _key in _group:
            _kind = next((k for k in ('explosion','core','outer','burn') if k in _key), 'body')
            FIELD_LABELS.setdefault(_key, _color_labels[_kind])

HELP_TEXT.update(rows("""
grenade_model|Seleziona solo l'aspetto del proiettile sperimentale. Ogni modello conserva i propri colori. Il missile segue la traiettoria senza ruotare su se stesso.|Choisit uniquement l'apparence du projectile expérimental. Chaque modèle conserve ses couleurs. Le missile suit sa trajectoire sans tournoyer.|Wählt nur das Aussehen des experimentellen Projektils. Jedes Modell behält seine Farben. Die Rakete folgt ihrer Flugbahn ohne zu wirbeln.|Selecciona solo la apariencia del proyectil experimental. Cada modelo conserva sus colores. El misil sigue la trayectoria sin girar sobre sí mismo.
flame_mine_model|Seleziona il modello della mina sperimentale. Ogni modello conserva separatamente i colori del corpo e delle fiamme. Danni e comportamento della mina restano gli stessi.|Choisit le modèle de mine expérimentale. Chaque modèle conserve séparément les couleurs du corps et des flammes. Les dégâts et le comportement restent identiques.|Wählt das Modell der experimentellen Mine. Jedes Modell behält eigene Körper- und Flammenfarben. Schaden und Verhalten bleiben gleich.|Selecciona el modelo de mina experimental. Cada modelo conserva por separado los colores del cuerpo y de las llamas. El daño y el comportamiento no cambian.
"""))

TEXT.update(rows("""
Encounter difficulty and Nanotech|Difficoltà e Nanotech|Difficulté et Nanotech|Schwierigkeit und Nanotech|Dificultad y Nanotech
"""))
FIELD_LABELS.update({'enemy_health_multiplier': {'en': 'Enemy health multiplier', 'it': 'Moltiplicatore vita nemici', 'fr': 'Multiplicateur de santé ennemie', 'de': 'Gegner-Lebensmultiplikator', 'es': 'Multiplicador de salud enemiga'}, 'enemy_health_include_bosses': {'en': 'Apply health multiplier to bosses', 'it': 'Applica vita ai boss', 'fr': 'Appliquer la santé aux boss', 'de': 'Lebensmultiplikator für Bosse', 'es': 'Aplicar salud a jefes'}, 'extra_enemies_per_level': {'en': 'Additional enemies per planet', 'it': 'Nemici aggiuntivi per pianeta', 'fr': 'Ennemis supplémentaires par planète', 'de': 'Zusätzliche Gegner pro Planet', 'es': 'Enemigos adicionales por planeta'}, 'extra_enemy_prefer_small': {'en': 'Prioritize small enemies', 'it': 'Priorità nemici piccoli', 'fr': 'Priorité aux petits ennemis', 'de': 'Kleine Gegner bevorzugen', 'es': 'Priorizar enemigos pequeños'}, 'extra_enemy_prefer_medium': {'en': 'Prioritize medium enemies', 'it': 'Priorità nemici medi', 'fr': 'Priorité aux ennemis moyens', 'de': 'Mittlere Gegner bevorzugen', 'es': 'Priorizar enemigos medianos'}, 'extra_enemy_prefer_large': {'en': 'Prioritize large enemies', 'it': 'Priorità nemici grandi', 'fr': 'Priorité aux grands ennemis', 'de': 'Große Gegner bevorzugen', 'es': 'Priorizar enemigos grandes'}, 'enemy_damage_by_health': {'en': 'Enemy damage tiers and Nanotech progression', 'it': 'Danni per classe e progressione Nanotech', 'fr': 'Dégâts par catégorie et progression Nanotech', 'de': 'Gegnerschaden und Nanotech-Fortschritt', 'es': 'Daño por categoría y progreso Nanotech'}, 'enemy_damage_multiplier': {'en': 'Enemy damage multiplier', 'it': 'Moltiplicatore danno nemici', 'fr': 'Multiplicateur des dégâts ennemis', 'de': 'Gegner-Schadensmultiplikator', 'es': 'Multiplicador de daño enemigo'}, 'enemy_perception_multiplier': {'en': 'Enemy perception multiplier', 'it': 'Moltiplicatore percezione nemici', 'fr': 'Multiplicateur de perception ennemie', 'de': 'Gegner-Wahrnehmungsmultiplikator', 'es': 'Multiplicador de percepción enemiga'},  'nanotech_xp_per_upgrade': {'en': 'XP per Nanotech upgrade', 'it': 'XP per potenziamento Nanotech', 'fr': 'XP par amélioration Nanotech', 'de': 'EP pro Nanotech-Verbesserung', 'es': 'XP por mejora Nanotech'}, 'nanotech_max_health': {'en': 'Maximum Nanotech from XP', 'it': 'Nanotech massimo da XP', 'fr': 'Nanotech maximal par XP', 'de': 'Maximales Nanotech durch EP', 'es': 'Nanotech máximo por XP'}, 'nanotech_upgrade_sound': {'en': 'Nanotech upgrade notification sound', 'it': 'Suono potenziamento Nanotech', 'fr': 'Son de notification Nanotech', 'de': 'Nanotech-Benachrichtigungston', 'es': 'Sonido de mejora Nanotech'}, 'nanotech_random_healing': {'en': 'Random Nanotech healing (1-4)', 'it': 'Cura Nanotech casuale (1-4)', 'fr': 'Soins Nanotech aléatoires (1-4)', 'de': 'Zufällige Nanotech-Heilung (1-4)', 'es': 'Curación Nanotech aleatoria (1-4)'}, })
HELP_TEXT['enemy_health_multiplier']={'en': 'Scales effective enemy HP and actual-loss XP together. Native AI phase thresholds are preserved.', 'it': 'Moltiplicatore vita nemici. Intervallo: 0.1–100. Impostazione globale salvata nel profilo.', 'fr': 'Multiplicateur de santé ennemie. Plage : 0.1–100. Paramètre global enregistré dans le profil.', 'de': 'Gegner-Lebensmultiplikator. Bereich: 0.1–100. Globale Einstellung im Profil gespeichert.', 'es': 'Multiplicador de salud enemiga. Intervalo: 0.1–100. Ajuste global guardado en el perfil.'}
HELP_TEXT['enemy_health_include_bosses']={'en': 'Includes the four boss classes in health scaling.', 'it': 'Applica vita ai boss. Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Appliquer la santé aux boss. Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Lebensmultiplikator für Bosse. Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Aplicar salud a jefes. Intervalo: Off / On. Ajuste global guardado en el perfil.'}
HELP_TEXT['extra_enemies_per_level']={'en': 'Total additional enemies per level load, not per original enemy. Unsafe placements are skipped.', 'it': 'Nemici aggiuntivi per pianeta. Intervallo: 0–100. Impostazione globale salvata nel profilo.', 'fr': 'Ennemis supplémentaires par planète. Plage : 0–100. Paramètre global enregistré dans le profil.', 'de': 'Zusätzliche Gegner pro Planet. Bereich: 0–100. Globale Einstellung im Profil gespeichert.', 'es': 'Enemigos adicionales por planeta. Intervalo: 0–100. Ajuste global guardado en el perfil.'}
HELP_TEXT['extra_enemy_prefer_small']={'en': 'Prefer enemies with 100 original HP, before health scaling.', 'it': 'Priorità nemici piccoli. Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Priorité aux petits ennemis. Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Kleine Gegner bevorzugen. Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Priorizar enemigos pequeños. Intervalo: Off / On. Ajuste global guardado en el perfil.'}
HELP_TEXT['extra_enemy_prefer_medium']={'en': 'Prefer enemies with 200 original HP, before health scaling.', 'it': 'Priorità nemici medi. Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Priorité aux ennemis moyens. Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Mittlere Gegner bevorzugen. Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Priorizar enemigos medianos. Intervalo: Off / On. Ajuste global guardado en el perfil.'}
HELP_TEXT['extra_enemy_prefer_large']={'en': 'Prefer enemies with at least 300 original HP, before health scaling.', 'it': 'Priorità nemici grandi. Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Priorité aux grands ennemis. Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Große Gegner bevorzugen. Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Priorizar enemigos grandes. Intervalo: Off / On. Ajuste global guardado en el perfil.'}
HELP_TEXT['enemy_damage_by_health']={'en': 'Enemies with 100/200/300+ original HP remove 1/2/3 Nanotech. Enables hidden Ratchet XP from accepted weapon XP. Environmental damage stays native.', 'it': 'Danni per classe e progressione Nanotech. Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Dégâts par catégorie et progression Nanotech. Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Gegnerschaden und Nanotech-Fortschritt. Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Daño por categoría y progreso Nanotech. Intervalo: Off / On. Ajuste global guardado en el perfil.'}
HELP_TEXT['enemy_damage_multiplier']={'en': 'Multiplies health-tier damage, rounded to 1-3 Nanotech. Requires enemy damage tiers.', 'it': 'Moltiplicatore danno nemici. Intervallo: 0.1–3. Impostazione globale salvata nel profilo.', 'fr': 'Multiplicateur des dégâts ennemis. Plage : 0.1–3. Paramètre global enregistré dans le profil.', 'de': 'Gegner-Schadensmultiplikator. Bereich: 0.1–3. Globale Einstellung im Profil gespeichert.', 'es': 'Multiplicador de daño enemigo. Intervalo: 0.1–3. Ajuste global guardado en el perfil.'}
HELP_TEXT['enemy_perception_multiplier']={'en': 'Scales enemy detection and engagement distances. Does not change projectile physics.', 'it': 'Moltiplicatore percezione nemici. Intervallo: 0.1–10. Impostazione globale salvata nel profilo.', 'fr': 'Multiplicateur de perception ennemie. Plage : 0.1–10. Paramètre global enregistré dans le profil.', 'de': 'Gegner-Wahrnehmungsmultiplikator. Bereich: 0.1–10. Globale Einstellung im Profil gespeichert.', 'es': 'Multiplicador de percepción enemiga. Intervalo: 0.1–10. Ajuste global guardado en el perfil.'}
HELP_TEXT['nanotech_xp_per_upgrade']={'en': 'Constant hidden XP threshold. Each completion adds one usable maximum HP; excess XP carries over. Max-grade weapons still contribute.', 'it': 'XP per potenziamento Nanotech. Intervallo: 0.1–1000000. Impostazione globale salvata nel profilo.', 'fr': 'XP par amélioration Nanotech. Plage : 0.1–1000000. Paramètre global enregistré dans le profil.', 'de': 'EP pro Nanotech-Verbesserung. Bereich: 0.1–1000000. Globale Einstellung im Profil gespeichert.', 'es': 'XP por mejora Nanotech. Intervalo: 0.1–1000000. Ajuste global guardado en el perfil.'}
HELP_TEXT['nanotech_max_health']={'en': 'Maximum health from progression. Native Premium/Ultra upgrades are never removed. Progress is stored per save.', 'it': 'Nanotech massimo da XP. Intervallo: 4–64. Impostazione globale salvata nel profilo.', 'fr': 'Nanotech maximal par XP. Plage : 4–64. Paramètre global enregistré dans le profil.', 'de': 'Maximales Nanotech durch EP. Bereich: 4–64. Globale Einstellung im Profil gespeichert.', 'es': 'Nanotech máximo por XP. Intervalo: 4–64. Ajuste global guardado en el perfil.'}
HELP_TEXT['nanotech_upgrade_sound']={'en': 'Play the HelpDesk notification sound for Max Nanotech increased! Independent of weapon upgrade sound.', 'it': 'Suono potenziamento Nanotech. Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Son de notification Nanotech. Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Nanotech-Benachrichtigungston. Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Sonido de mejora Nanotech. Intervalo: Off / On. Ajuste global guardado en el perfil.'}
HELP_TEXT['nanotech_random_healing']={'en': 'Requires damage tiers. Each field pickup heals 1-4 HP: original, indigo, violet, red. Never overheals or revives Ratchet.', 'it': 'Cura Nanotech casuale (1-4). Intervallo: Off / On. Impostazione globale salvata nel profilo.', 'fr': 'Soins Nanotech aléatoires (1-4). Plage : Off / On. Paramètre global enregistré dans le profil.', 'de': 'Zufällige Nanotech-Heilung (1-4). Bereich: Off / On. Globale Einstellung im Profil gespeichert.', 'es': 'Curación Nanotech aleatoria (1-4). Intervalo: Off / On. Ajuste global guardado en el perfil.'}


# Lava controls share the launcher's supported languages; technical INI keys stay stable.
FIELD_LABELS.update(rows('''
lava_stream_enabled|Getto di lava sperimentale|Jet de lave expérimental|Experimenteller Lavastrom|Chorro de lava experimental
lava_launch_speed|Velocità iniziale della lava (unità/s)|Vitesse initiale de la lave (unités/s)|Lava-Startgeschwindigkeit (Einheiten/s)|Velocidad inicial de lava (unidades/s)
lava_gravity|Gravità della lava (unità/s²)|Gravité de la lave (unités/s²)|Lava-Schwerkraft (Einheiten/s²)|Gravedad de lava (unidades/s²)
lava_max_range|Distanza massima percorsa dalla lava|Distance maximale parcourue par la lave|Maximale Lava-Flugstrecke|Distancia máxima recorrida por la lava
lava_size_percent|Spessore del getto di lava (%)|Épaisseur du jet de lave (%)|Dicke des Lavastroms (%)|Grosor del chorro de lava (%)
lava_core_color|Colore del nucleo della lava|Couleur du cœur de la lave|Farbe des Lavakerns|Color del núcleo de lava
lava_outer_color|Colore esterno della lava|Couleur extérieure de la lave|Äußere Lavafarbe|Color exterior de lava
lava_impact_color|Colore dell'impatto della lava|Couleur de l'impact de la lave|Farbe des Lavaaufpralls|Color del impacto de lava
lava_light_color|Colore della luce della lava|Couleur de la lumière de la lave|Farbe des Lavalichts|Color de la luz de lava
'''))
HELP_TEXT.update(rows('''
lava_stream_enabled|Sostituisce le fiamme normali e triple con un getto di lava ad arco. Se entrambi gli esperimenti sono attivi, ha priorità la lava. Usa danni al secondo, intervalli di danno e munizioni e munizioni massime sperimentali di questo grado. Le gocce sovrapposte non moltiplicano i danni.|Remplace les flammes normales et triples par un jet de lave en arc. Si les deux expériences sont actives, la lave est prioritaire. Utilise les dégâts par seconde, les intervalles de dégâts et de munitions et les munitions maximales expérimentales de ce grade. Les gouttes superposées ne multiplient pas les dégâts.|Ersetzt normale und dreifache Flammen durch einen bogenförmigen Lavastrom. Sind beide Experimente aktiv, hat Lava Vorrang. Verwendet Schaden pro Sekunde, Schadens- und Munitionsintervalle sowie die experimentelle Munitionskapazität dieser Stufe. Überlappende Tropfen vervielfachen den Schaden nicht.|Sustituye las llamas normales y triples por un chorro de lava en arco. Si ambos experimentos están activos, la lava tiene prioridad. Usa el daño por segundo, los intervalos de daño y munición y la capacidad experimental de este grado. Las gotas superpuestas no multiplican el daño.
lava_launch_speed|Velocità iniziale in unità al secondo. La gravità incurva il getto dopo il lancio. Non modifica danni o consumo di munizioni.|Vitesse initiale en unités par seconde. La gravité courbe le jet après le lancement. Ne modifie ni les dégâts ni la consommation de munitions.|Startgeschwindigkeit in Einheiten pro Sekunde. Die Schwerkraft krümmt den Strom nach dem Start. Ändert weder Schaden noch Munitionsverbrauch.|Velocidad inicial en unidades por segundo. La gravedad curva el chorro después del lanzamiento. No cambia el daño ni el consumo de munición.
lava_gravity|Accelerazione verso il basso in unità al secondo quadrato. 0 produce un getto dritto.|Accélération vers le bas en unités par seconde carrée. 0 produit un jet droit.|Abwärtsbeschleunigung in Einheiten pro Sekunde zum Quadrat. 0 erzeugt einen geraden Strom.|Aceleración hacia abajo en unidades por segundo al cuadrado. 0 produce un chorro recto.
lava_max_range|Lunghezza massima della traiettoria percorsa, non distanza orizzontale. Il getto si ferma all'impatto o al raggiungimento di questa distanza. Ogni goccia ha anche una durata massima di sicurezza di 2 secondi.|Longueur maximale de la trajectoire parcourue, et non distance horizontale. Le jet s'arrête à l'impact ou à cette distance. Chaque goutte a aussi une durée de sécurité maximale de 2 secondes.|Maximale Länge der zurückgelegten Flugbahn, nicht die horizontale Entfernung. Der Strom endet beim Aufprall oder an dieser Strecke. Jeder Tropfen hat außerdem eine Sicherheitslebensdauer von höchstens 2 Sekunden.|Longitud máxima de la trayectoria recorrida, no distancia horizontal. El chorro termina al impactar o alcanzar esta distancia. Cada gota también tiene una duración máxima de seguridad de 2 segundos.
lava_size_percent|Modifica insieme spessore visivo e raggio che infligge danni. 100 corrisponde a un raggio di contatto di 0,22 unità. Non modifica i danni al secondo.|Modifie ensemble l'épaisseur visuelle et le rayon de dégâts. 100 correspond à un rayon de contact de 0,22 unité. Ne modifie pas les dégâts par seconde.|Ändert sichtbare Dicke und Schadensradius gemeinsam. 100 entspricht einem Kontaktradius von 0,22 Einheiten. Ändert den Schaden pro Sekunde nicht.|Cambia a la vez el grosor visual y el radio de daño. 100 corresponde a un radio de contacto de 0,22 unidades. No cambia el daño por segundo.
'''))


# WEAPON EVOLUTION V65 LAVA ELEVATION
FIELD_LABELS.update(rows('''
lava_launch_angle_degrees|Angolo verticale iniziale della lava (gradi)|Angle vertical initial de la lave (degrés)|Lava-Startwinkel nach oben (Grad)|Ángulo vertical inicial de lava (grados)
'''))
HELP_TEXT.update(rows('''
lava_launch_angle_degrees|Angolo aggiunto alla mira dell'arma. 0 segue la mira; valori positivi alzano il getto, valori negativi lo abbassano. Valore iniziale: 10 gradi. Si arresta alla verticale senza invertire la direzione. Non modifica velocità, danni o consumo di munizioni.|Angle ajouté à la visée de l'arme. 0 suit la visée ; les valeurs positives relèvent le jet et les valeurs négatives l'abaissent. Valeur initiale : 10 degrés. Limité à la verticale sans inverser la direction. Ne modifie ni vitesse, ni dégâts, ni consommation de munitions.|Zusätzlicher Winkel zur Waffenvisierung. 0 folgt dem Ziel; positive Werte heben den Strom, negative senken ihn. Standard: 10 Grad. An der Vertikalen begrenzt, ohne die Richtung umzukehren. Ändert weder Geschwindigkeit noch Schaden oder Munitionsverbrauch.|Ángulo añadido a la mira del arma. 0 sigue la mira; los valores positivos elevan el chorro y los negativos lo bajan. Valor inicial: 10 grados. Limitado a la vertical sin invertir la dirección. No cambia velocidad, daño ni consumo de munición.
'''))

# Weapon Evolution v66: upgrade ammunition reward
FIELD_LABELS['refill_ammo_on_upgrade']={'en': 'Refill ammo on weapon upgrade', 'it': 'Ricarica munizioni al potenziamento', 'fr': 'Recharger au changement de niveau', 'de': 'Munition bei Waffenaufstieg auffuellen', 'es': 'Recargar al mejorar el arma'}
HELP_TEXT['refill_ammo_on_upgrade']={'en': 'Refills an owned ammo weapon to the newly earned grade capacity, including its active experimental capacity. Loading a save grants no ammo. Suck Cannon contents are not created.', 'it': 'Ricarica le armi possedute al massimo del grado appena raggiunto, anche sperimentale. Caricare un salvataggio non concede munizioni. Non crea contenuti per il cannone a risucchio.', 'fr': 'Recharge les armes possedees au maximum du nouveau niveau, y compris experimental. Charger une sauvegarde ne donne aucune munition. Ne cree pas de projectiles pour le Canon aspirateur.', 'de': 'Fuellt eine eigene Munitionswaffe bis zur Kapazitaet der neuen Stufe, auch experimentell. Laden eines Spielstands gibt keine Munition. Erzeugt keinen Inhalt fuer die Saugkanone.', 'es': 'Recarga las armas propias hasta el maximo del nuevo nivel, incluido el experimental. Cargar una partida no da municion. No crea proyectiles para el Succionador.'}


# Weapon Evolution v67: seeded Gold Bolt rewards.
def _v67_add_translations(table,translations):
    languages=set(translations)
    by_language=any(str(key).lower() in languages for key in table)
    if not by_language:
        by_language=not any(isinstance(value,dict) and any(str(key).lower() in languages for key in value) for value in table.values())
    for language,values in translations.items():
        if by_language:
            actual=next((key for key in table if str(key).lower()==language),language)
            table.setdefault(actual,{}).update(values)
        else:
            for key,value in values.items():
                entry=table.setdefault(key,{})
                actual=next((key for key in entry if str(key).lower()==language),language)
                entry[actual]=value
_v67_add_translations(FIELD_LABELS,{'en': {'golden_bolts_to_weapons': 'Golden Bolts_to_Weapons', 'golden_bolts_early_bias_percent': 'Early weapon placement bias (%)'}, 'it': {'golden_bolts_to_weapons': 'Golden Bolts_to_Weapons', 'golden_bolts_early_bias_percent': 'Preferenza per le posizioni iniziali (%)'}, 'fr': {'golden_bolts_to_weapons': 'Golden Bolts_to_Weapons', 'golden_bolts_early_bias_percent': 'Priorité aux emplacements accessibles tôt (%)'}, 'de': {'golden_bolts_to_weapons': 'Golden Bolts_to_Weapons', 'golden_bolts_early_bias_percent': 'Bevorzugung früher Waffenfundorte (%)'}, 'es': {'golden_bolts_to_weapons': 'Golden Bolts_to_Weapons', 'golden_bolts_early_bias_percent': 'Preferencia por ubicaciones tempranas (%)'}})
_v67_add_translations(HELP_TEXT,{'en': {'golden_bolts_to_weapons': 'Replace 15 Gold Bolts with the 15 weapons. A new game generates a layout saved with that slot. Uses a separate memory card and refuses ordinary saves. Original weapon sources remain available; this option changes Gold Bolt rewards only. Gemlik’s extra RYNO pickup is replaced by the seeded reward system.', 'golden_bolts_early_bias_percent': '0: locations anywhere, subject to access rules. 100: the earliest conservative gadget-access tiers; late planets have no weapon replacements. Existing saves keep their original layout and bias. Range: 0–100.'}, 'it': {'golden_bolts_to_weapons': 'Sostituisce 15 bolt d’oro con le 15 armi. Ogni nuova partita genera scambi salvati nello slot. Usa una scheda separata e rifiuta i salvataggi normali. Le fonti originali delle armi restano disponibili. La ricompensa RYNO aggiuntiva a Gemlik è sostituita dal sistema casuale.', 'golden_bolts_early_bias_percent': '0: posizioni ovunque, rispettando i requisiti di accesso. 100: posizioni dei primi gruppi accessibili con i gadget; nessuno scambio sui pianeti finali. I salvataggi esistenti mantengono la distribuzione iniziale. Intervallo: 0–100.'}, 'fr': {'golden_bolts_to_weapons': 'Remplace 15 boulons en or par les 15 armes. Une nouvelle partie génère une répartition conservée dans la sauvegarde. Carte mémoire séparée, sauvegardes ordinaires refusées. Les sources originales des armes restent disponibles. Le RYNO supplémentaire de Gemlik est remplacé par ce système.', 'golden_bolts_early_bias_percent': '0 : emplacements libres selon les règles d’accès. 100 : premiers groupes accessibles avec les gadgets, sans armes sur les dernières planètes. Une sauvegarde conserve sa répartition initiale. Valeurs : 0–100.'}, 'de': {'golden_bolts_to_weapons': 'Ersetzt 15 Gold-Bolts durch die 15 Waffen. Jedes neue Spiel erzeugt eine im Spielstand gespeicherte Verteilung. Separate Speicherkarte; normale Spielstände werden abgelehnt. Ursprüngliche Waffenquellen bleiben verfügbar. Der zusätzliche RYNO auf Gemlik wird durch dieses System ersetzt.', 'golden_bolts_early_bias_percent': '0: beliebige zulässige Fundorte. 100: die frühesten konservativen Gadget-Zugangsgruppen, ohne Waffen auf späten Planeten. Bestehende Spielstände behalten ihre Verteilung. Bereich: 0–100.'}, 'es': {'golden_bolts_to_weapons': 'Sustituye 15 guitones de oro por las 15 armas. Una partida nueva genera una distribución guardada con la partida. Usa una tarjeta separada y rechaza partidas normales. Las fuentes originales de armas siguen disponibles. El RYNO adicional de Gemlik se sustituye por este sistema.', 'golden_bolts_early_bias_percent': '0: cualquier ubicación permitida por las reglas de acceso. 100: primeros grupos accesibles con los dispositivos, sin armas en los últimos planetas. Las partidas existentes conservan su distribución. Intervalo: 0–100.'}})

# Nanotech progression HUD. Stable INI keys remain identical in every language.
FIELD_LABELS['nanotech_xp_bar_opacity_percent'] = {
    'en': 'Nanotech XP bar opacity (%)',
    'it': 'Opacità barra XP Nanotech (%)',
    'fr': "Opacité de la barre d'XP Nanotech (%)",
    'de': 'Deckkraft der Nanotech-EP-Leiste (%)',
    'es': 'Opacidad de la barra de XP Nanotech (%)',
}
HELP_TEXT['nanotech_xp_bar_opacity_percent'] = {
    'en': 'Only the Nanotech progression bar above the health panel. 0 hides it; 100 uses full opacity. Weapon selection activates it, then it fades with the Nanotech HUD. Damage and healing do not activate it. Requires enemy damage tiers and Nanotech progression.',
    'it': 'Solo la barra di progressione Nanotech sopra il riquadro della vita. 0 la nasconde; 100 è opacità piena. Appare con la selezione armi, poi scompare con il Nanotech. Danni e cure non la attivano. Richiede danni per classe e progressione Nanotech.',
    'fr': "Uniquement la barre de progression Nanotech au-dessus de la santé. 0 la masque ; 100 est opaque. La sélection des armes l'active, puis elle disparaît avec le Nanotech. Les dégâts et les soins ne l'activent pas. Nécessite les dégâts par catégorie et la progression Nanotech.",
    'de': 'Nur die Nanotech-Fortschrittsleiste über der Gesundheitsanzeige. 0 blendet sie aus; 100 ist volle Deckkraft. Die Waffenauswahl aktiviert sie; sie verschwindet mit der Nanotech-Anzeige. Schaden und Heilung aktivieren sie nicht. Erfordert Gegnerschaden nach Klassen und Nanotech-Fortschritt.',
    'es': 'Solo la barra de progreso Nanotech sobre la salud. 0 la oculta; 100 es opacidad completa. La selección de armas la activa y desaparece con el Nanotech. El daño y la curación no la activan. Requiere daño por categoría y progreso Nanotech.',
}


FIELD_LABELS.update(rows("""
enemy_speed_multiplier|Moltiplicatore velocità azioni nemici|Vitesse des actions ennemies|Aktionstempo der Gegner|Velocidad de las acciones enemigas
enemy_projectile_range_multiplier|Moltiplicatore portata proiettili nemici|Portée des projectiles ennemis|Reichweite feindlicher Geschosse|Alcance de proyectiles enemigos
enemy_projectile_speed_multiplier|Moltiplicatore velocità proiettili nemici|Vitesse des projectiles ennemis|Geschwindigkeit feindlicher Geschosse|Velocidad de proyectiles enemigos
golden_bolts_one_weapon_per_planet|Al massimo un'arma per pianeta|Une seule arme par planète au maximum|Höchstens eine Waffe pro Planet|Como máximo un arma por planeta
"""))
HELP_TEXT.update(rows("""
enemy_speed_multiplier|Movimenti, animazioni e azioni di nemici e boss. 1 = normale, 2 = doppia velocità. Collisioni controllate ad ogni passo. Non accelera Ratchet, gli alleati o i filmati. Intervallo: 0.1-4.|Mouvements, animations et actions des ennemis et boss. 1 = normal, 2 = vitesse double. Collisions vérifiées à chaque pas. Ratchet, alliés et cinématiques inchangés. Plage : 0.1-4.|Bewegungen, Animationen und Aktionen von Gegnern und Bossen. 1 = normal, 2 = doppelt. Kollisionsprüfung bei jedem Schritt. Ratchet, Verbündete und Zwischensequenzen bleiben unverändert. Bereich: 0.1-4.|Movimientos, animaciones y acciones de enemigos y jefes. 1 = normal, 2 = doble. Colisiones comprobadas en cada paso. No acelera a Ratchet, aliados ni escenas. Rango: 0.1-4.
enemy_projectile_speed_multiplier|Velocità di colpi, bombe, missili e proiettili dei boss. Indipendente da velocità nemici e portata. Ogni passo controlla le collisioni. Accelera anche gravità e micce delle traiettorie balistiche. Intervallo: 0.1-4.|Vitesse des tirs, bombes, missiles et projectiles des boss. Indépendante des ennemis et de la portée. Collisions à chaque pas. Accélère aussi la gravité et les mèches balistiques. Plage : 0.1-4.|Tempo feindlicher Schüsse, Bomben, Raketen und Bossgeschosse. Unabhängig von Gegnergeschwindigkeit und Reichweite. Kollisionsprüfung je Schritt. Auch Schwerkraft und Zünder laufen schneller. Bereich: 0.1-4.|Velocidad de disparos, bombas, misiles y proyectiles de jefes. Independiente de enemigos y alcance. Colisiones en cada paso. También acelera gravedad y mechas balísticas. Rango: 0.1-4.
enemy_projectile_range_multiplier|Modifica portata, durata dei colpi dritti o distanza orizzontale dei colpi balistici. Ostacoli, punti bersaglio fissi e confini del mondo restano validi. Danni ed esplosioni invariati. Intervallo: 0.1-10.|Modifie portée, durée des tirs droits ou distance horizontale balistique. Obstacles, cibles fixes et limites du monde restent actifs. Dégâts et explosions inchangés. Plage : 0.1-10.|Skaliert Reichweite, Lebensdauer gerader Schüsse oder horizontale ballistische Flugweite. Hindernisse, feste Zielpunkte und Weltgrenzen gelten weiter. Schaden und Explosionen unverändert. Bereich: 0.1-10.|Modifica alcance, duración de disparos rectos o distancia horizontal balística. Se conservan obstáculos, objetivos fijos y límites del mundo. Daño y explosiones sin cambios. Rango: 0.1-10.
golden_bolts_one_weapon_per_planet|Solo nuove partite: le 15 armi sono distribuite su 15 pianeti diversi, una location per pianeta. Restano i vincoli di accesso per esplosivi, Visibomba e Provocatore. I salvataggi esistenti mantengono le loro posizioni.|Nouvelles parties : les 15 armes sur 15 planètes différentes, un emplacement par planète. Contraintes des explosifs, du Visi-bomb et du Provocateur conservées. Les sauvegardes existantes gardent leur répartition.|Nur neue Spiele: 15 Waffen auf 15 verschiedenen Planeten, ein Fundort pro Planet. Zugangsregeln für Sprengstoff, Visibomb und Provokator bleiben bestehen. Vorhandene Spielstände behalten ihre Verteilung.|Solo partidas nuevas: 15 armas en 15 planetas distintos, una ubicación por planeta. Se conservan las restricciones de explosivos, Visibomba y Provocador. Las partidas existentes mantienen sus ubicaciones.
"""))

FIELD_LABELS['enemy_chase_range_multiplier']={
'en':'Enemy chase range multiplier (extra)',
'it':'Moltiplicatore aggiuntivo distanza inseguimento',
'fr':'Multiplicateur supplémentaire de poursuite',
'de':'Zusätzlicher Verfolgungsreichweiten-Multiplikator',
'es':'Multiplicador adicional de persecución'}
HELP_TEXT['enemy_chase_range_multiplier']={
'en':'Additional multiplier for home and territory limits after perception scaling. 1 follows perception; perception 2 and chase 2 give four times the native territory. Does not enlarge melee attacks or remove physical walls and scripted routes. Range: 0.1-10.',
'it':'Moltiplica ulteriormente i limiti di inseguimento dopo la percezione. 1 segue la percezione; percezione 2 e inseguimento 2 danno un territorio quattro volte maggiore. Non modifica i colpi corpo a corpo, i muri o i percorsi obbligati. Intervallo: 0.1-10.',
'fr':'Multiplie les limites de poursuite après la perception. Perception 2 et poursuite 2 donnent un territoire quatre fois plus grand. Ne change pas les attaques de mêlée, les murs ni les itinéraires imposés. Plage : 0.1-10.',
'de':'Zusätzlicher Faktor für Verfolgungsgrenzen nach der Wahrnehmung. Wahrnehmung 2 und Verfolgung 2 ergeben die vierfache Reichweite. Nahkampftreffer, Wände und vorgegebene Routen bleiben erhalten. Bereich: 0.1-10.',
'es':'Multiplica los límites de persecución después de la percepción. Percepción 2 y persecución 2 dan un territorio cuatro veces mayor. No cambia golpes cuerpo a cuerpo, muros ni rutas obligatorias. Rango: 0.1-10.'}

# v75: XP health is an additive allowance; native purchases keep their own slots.
FIELD_LABELS['nanotech_max_health'] = dict(zip(LANGUAGES, (
    'Maximum extra Nanotech from XP', 'Nanotech aggiuntivo massimo da XP',
    'Nanotech supplémentaire maximal par XP', 'Maximales zusätzliches Nanotech durch EP',
    'Nanotech adicional máximo por XP')))
HELP_TEXT['nanotech_max_health'] = dict(zip(LANGUAGES, (
    'Limits XP-earned extra HP only (0-64). Total HP = 4 + Premium (1) + Ultra (3) + XP bonus. Lowering this limit preserves earned progress.',
    'Limita solo i punti vita aggiuntivi da XP (0-64). Vita totale = 4 + Premium (1) + Ultra (3) + bonus XP. Ridurre il limite conserva i progressi ottenuti.',
    'Limite uniquement les PV supplémentaires gagnés par XP (0-64). PV totaux = 4 + Premium (1) + Ultra (3) + bonus XP. Réduire la limite conserve la progression.',
    'Begrenzt nur zusätzliche EP-Lebenspunkte (0-64). Gesamtleben = 4 + Premium (1) + Ultra (3) + EP-Bonus. Ein niedrigeres Limit erhält den Fortschritt.',
    'Limita solo los PV adicionales ganados por XP (0-64). PV totales = 4 + Premium (1) + Ultra (3) + bonus XP. Reducir el límite conserva el progreso.')))
HELP_TEXT['enemy_damage_multiplier'] = dict(zip(LANGUAGES, (
    'Scales health-based enemy damage and all boss attack profiles (0.1-100). Rounded to whole HP, minimum 1.',
    'Moltiplica i danni per classe e tutti gli attacchi dei boss (0.1-100). Arrotondamento a punti vita interi, minimo 1.',
    'Multiplie les dégâts par catégorie et toutes les attaques des boss (0.1-100). Arrondi aux PV entiers, minimum 1.',
    'Multipliziert klassifizierten Gegnerschaden und alle Bossangriffe (0.1-100). Auf ganze Lebenspunkte gerundet, mindestens 1.',
    'Multiplica el daño por categoría y todos los ataques de jefes (0.1-100). Redondea a PV enteros, mínimo 1.')))
HELP_TEXT['enemy_damage_by_health'] = dict(zip(LANGUAGES, (
    'Enables uncapped damage tiers from original enemy HP, attack-specific boss damage and XP-earned extra Nanotech. Premium adds 1 HP and Ultra adds 3 HP independently.',
    'Attiva danni senza limite basati sugli HP originali, danni specifici per attacco dei boss e Nanotech aggiuntivo da XP. Premium aggiunge 1 punto vita e Ultra 3, separatamente.',
    'Active les dégâts sans plafond selon les PV originaux, les dégâts propres aux attaques des boss et le Nanotech par XP. Premium ajoute 1 PV et Ultra 3, indépendamment.',
    'Aktiviert unbegrenzte Schadensstufen nach ursprünglichen Gegner-HP, angriffsspezifischen Bossschaden und EP-Nanotech. Premium fügt 1 und Ultra 3 Lebenspunkte unabhängig hinzu.',
    'Activa daño sin límite según los PV originales, daño específico de cada ataque de jefe y Nanotech por XP. Premium añade 1 PV y Ultra 3 de forma independiente.')))


TEXT.update(rows("""
All eight grades are on this page. Scroll to see them all. Right-click a value to copy it to the other grades.|Tutti gli otto gradi sono in questa pagina. Scorri per visualizzarli. Clic destro su un valore per copiarlo negli altri gradi.|Les huit grades sont sur cette page. Faites défiler pour tous les voir. Clic droit pour copier une valeur aux autres grades.|Alle acht Stufen stehen auf dieser Seite. Scrollen zeigt alle. Rechtsklick kopiert einen Wert auf andere Stufen.|Los ocho grados están en esta página. Desplázate para verlos. Clic derecho para copiar un valor a los demás grados.
Value copied to all eight grades: |Valore copiato in tutti gli otto gradi: |Valeur copiée dans les huit grades : |Wert in alle acht Stufen kopiert: |Valor copiado a los ocho grados: 
"""))
FIELD_LABELS['hide_ryno']={language:'Hide_Ryno' for language in LANGUAGES}
HELP_TEXT['hide_ryno']=dict(zip(LANGUAGES,(
'Places the R.Y.N.O. in a secret area. Removes the Blackwater City seller without granting the weapon. Off keeps the native seller; seeded randomization takes priority.',
'Nasconde il MIPS in una zona segreta nel gioco. Rimuove il venditore di Blackwater City senza regalare l’arma. Off mantiene il venditore; la randomizzazione ha precedenza.',
'Place le TELT dans une zone secrète. Retire le vendeur de Blackwater City sans donner l’arme. Off conserve le vendeur ; la randomisation reste prioritaire.',
'Versteckt die R.I.D.A. in einem Geheimgebiet. Entfernt den Händler in Blackwater City, ohne die Waffe zu vergeben. Off behält den Händler; die Zufallsverteilung hat Vorrang.',
'Oculta el TAUN en una zona secreta. Retira al vendedor de Blackwater City sin entregar el arma. Off mantiene al vendedor; la aleatorización tiene prioridad.')))

TEXT.update(rows("""
Gold purchase (NG+)|Acquisto dorato (NG+)|Achat doré (NG+)|Goldkauf (NG+)|Compra dorada (NG+)
"""))


FIELD_LABELS['ngplus_multiplier'] = {language:'NGPlus_Multiplier' for language in LANGUAGES}
HELP_TEXT['ngplus_multiplier'] = dict(zip(LANGUAGES, (
    'NG+ only: multiplies enemy health, damage, action speed, projectile speed/range, perception and chase coefficients. Base 1 with 2 becomes 2; base 1.5 with 2 becomes 3. Normal games retain base values. Counts, switches and Nanotech XP requirements are unchanged. Range: 0.1-4.',
    'Solo NG+: moltiplica i moltiplicatori di vita, danno, velocità, velocità/distanza dei proiettili, percezione e inseguimento dei nemici. Base 1 con 2 diventa 2; base 1.5 con 2 diventa 3. Nel gioco normale restano i valori base. Quantità, interruttori e requisiti XP Nanotech non cambiano. Intervallo: 0.1-4.',
    'NG+ uniquement : multiplie les coefficients de vie, dégâts, vitesse, vitesse/portée des projectiles, perception et poursuite des ennemis. Base 1 avec 2 donne 2 ; base 1.5 avec 2 donne 3. La partie normale conserve les valeurs de base. Quantités, options et seuils XP Nanotech restent inchangés. Plage : 0.1-4.',
    'Nur NG+: multipliziert Gegner-Lebenspunkte, Schaden, Aktionsgeschwindigkeit, Projektiltempo/-reichweite, Wahrnehmung und Verfolgung. Basis 1 mit 2 ergibt 2; Basis 1.5 mit 2 ergibt 3. Im normalen Spiel gelten die Basiswerte. Anzahlen, Schalter und Nanotech-EP-Schwellen bleiben unverändert. Bereich: 0.1-4.',
    'Solo NG+: multiplica los factores de vida, daño, velocidad, velocidad/alcance de proyectiles, percepción y persecución de enemigos. Base 1 con 2 da 2; base 1.5 con 2 da 3. La partida normal conserva los valores base. Cantidades, opciones y requisitos XP Nanotech no cambian. Rango: 0.1-4.'
)))


HELP_TEXT['wrench_trail_color']=dict(zip(LANGUAGES, (
    'Melee trail color for this grade: default keeps the original pale blue; RRGGBB selects a custom color. The trail scales with the hit-radius multiplier captured when the attack starts.',
    'Colore della scia dei colpi per questo grado: default mantiene l’azzurro originale; RRGGBB sceglie un colore. La scia segue il moltiplicatore del raggio acquisito all’inizio dell’attacco.',
    'Couleur de la traînée des coups pour ce grade : default conserve le bleu clair ; RRGGBB choisit une couleur. La traînée suit le multiplicateur de rayon fixé au début de l’attaque.',
    'Farbe der Nahkampfspur dieser Stufe: default behält das ursprüngliche Hellblau; RRGGBB wählt eine Farbe. Die Spur folgt dem beim Angriffsbeginn erfassten Trefferradius-Multiplikator.',
    'Color de la estela de los golpes de este grado: default conserva el azul claro; RRGGBB elige un color. La estela sigue el multiplicador del radio fijado al empezar el ataque.'
)))

FIELD_LABELS['dpad_quick_select']=dict(zip(LANGUAGES,('D-pad Quick Select','Selezione rapida D-pad','Selection rapide D-pad','D-Pad-Schnellwahl','Seleccion rapida D-pad')))
FIELD_LABELS['r1_experimental']={language:'R1_Experimental' for language in LANGUAGES}
HELP_TEXT['dpad_quick_select']=dict(zip(LANGUAGES,(
    'Four physical D-pad equipment shortcuts, saved per game. Move with the left stick. In Weapons/Gadgets hold L1 plus a D-pad direction to assign the highlighted owned item; the cursor stays still. Up green, Right red, Down blue, Left pink. Native equipment restrictions still apply. Off restores D-pad movement.',
    'Quattro selezioni rapide sui tasti fisici del D-pad, salvate per partita. Muoviti con la levetta sinistra. In Armi/Gadget tieni L1 e premi una direzione per assegnare l\'oggetto posseduto evidenziato; il cursore rimane fermo. Su verde, destra rosso, giu blu, sinistra rosa. Restano i limiti nativi di equipaggiamento. Off ripristina il movimento con D-pad.',
    'Quatre raccourcis D-pad physiques, sauvegardes par partie. Deplacement au stick gauche. Dans Armes/Gadgets, maintenir L1 et une direction pour affecter l\'objet possede selectionne; le curseur reste fixe. Haut vert, droite rouge, bas bleu, gauche rose. Les restrictions natives restent actives. Off retablit le deplacement D-pad.',
    'Vier physische D-Pad-Ausruestungsplaetze pro Spielstand. Bewegung mit dem linken Stick. In Waffen/Gadgets L1 halten und eine Richtung druecken, um den eigenen markierten Gegenstand zuzuweisen; der Cursor bleibt stehen. Oben gruen, rechts rot, unten blau, links rosa. Native Ausruestungsgrenzen bleiben bestehen. Off stellt D-Pad-Bewegung wieder her.',
    'Cuatro accesos del D-pad fisico, guardados por partida. Movimiento con el stick izquierdo. En Armas/Gadgets mantene L1 y pulsa una direccion para asignar el objeto propio resaltado; el cursor queda fijo. Arriba verde, derecha rojo, abajo azul, izquierda rosa. Se respetan las restricciones originales. Off restaura el movimiento D-pad.',
)))
HELP_TEXT['r1_experimental']=dict(zip(LANGUAGES,(
    'Hold Triangle, select Pyrocitor, Mine Glove, Visibomb or R.Y.N.O. and press R1. Mode 0 native; 1 Lava Gun, flame mine, grenade launcher or continuous missiles; R.Y.N.O. mode 2 shotgun. Choices are saved per game, initially 0. Per-grade tuning and all other enhancements stay in the launcher/INI. Off restores launcher mode switches without erasing choices.',
    'Tieni Triangolo, seleziona Inceneritore, Guanto Mina, Visibomba o M.I.P.S. e premi R1. Mode 0 normale; 1 Lava Gun, mina lanciafiamme, lanciagranate o missili continui; M.I.P.S. mode 2 shotgun. Scelte salvate per partita, inizialmente 0. Valori per grado e tutti gli altri potenziamenti restano nel launcher/INI. Off ripristina gli interruttori del launcher senza cancellare le scelte.',
    'Maintenir Triangle, choisir Pyrolanceur, Gant a Mines, Visibombe ou T.E.L.T. et appuyer sur R1. Mode 0 normal; 1 Lava Gun, mine enflammee, lance-grenades ou missiles continus; T.E.L.T. mode 2 fusil. Choix sauvegardes par partie, initialement 0. Reglages par niveau et autres ameliorations restent dans le launcher/INI. Off retablit les options du launcher sans effacer les choix.',
    'Dreieck halten, Pyrozitor, Minen-Handschuh, Visibombe oder R.I.D.A. waehlen und R1 druecken. Mode 0 normal; 1 Lava Gun, Feuermine, Granatwerfer oder Dauerraketen; R.I.D.A. mode 2 Schrotflinte. Auswahl pro Spielstand, anfangs 0. Stufenwerte und andere Verbesserungen bleiben im Launcher/INI. Off stellt Launcher-Modi wieder her, ohne die Auswahl zu loeschen.',
    'Mantene Triangulo, selecciona Piromatic, Guante minas, Visicanon o T.A.U.N. y pulsa R1. Mode 0 normal; 1 Lava Gun, mina de fuego, lanzagranadas o misiles continuos; T.A.U.N. mode 2 escopeta. Opciones guardadas por partida, inicialmente 0. Ajustes por nivel y otras mejoras quedan en el launcher/INI. Off restaura los modos del launcher sin borrar las opciones.',
)))

# Repeated equipment shortcuts use owned native inventory slots only.
_DPAD_REPEAT_HELP={
    'en':'Press the same shortcut again to remove equipped boots or helmets. For a Clank pack, first equip the assigned pack, then cycle owned Heli-Pack, Thruster-Pack and Hydro-Pack on repeated presses.',
    'it':'Premi di nuovo la stessa scorciatoia per togliere scarpe o elmetti indossati. Per uno zaino di Clank, prima equipaggi quello assegnato, poi scorri tra Heli-Pack, Thruster-Pack e Hydro-Pack posseduti ripremendo il tasto.',
    'fr':'Appuyez encore sur le meme raccourci pour retirer les bottes ou le casque equipe. Pour un sac de Clank, equipez celui affecte, puis parcourez les Heli-Pack, Thruster-Pack et Hydro-Pack possedes.',
    'de':'Dieselbe Taste erneut druecken, um angelegte Stiefel oder Helme abzulegen. Bei Clanks Rucksack wird zuerst die Zuweisung angelegt; weitere Tastendruecke wechseln zwischen eigenen Heli-, Thruster- und Hydro-Packs.',
    'es':'Pulsa otra vez el mismo acceso para quitar las botas o el casco equipado. Para una mochila de Clank, equipa primero la asignada y despues alterna entre Heli-Pack, Thruster-Pack e Hydro-Pack que poseas.'}
for _language,_text in _DPAD_REPEAT_HELP.items():
    HELP_TEXT['dpad_quick_select'][_language]+=' '+_text
