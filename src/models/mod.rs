use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
    
// ── Struct mappant la table `utilisateurs` ──────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Utilisateur {
    pub id:            Uuid,
    pub email:         String,
    pub password_hash: String,
    pub nom:           String,
    pub prenom:        Option<String>,
    pub profession:    Option<String>,
    pub statut:        String,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

// ── Ce qu'on renvoie au client (sans le hash) ───────────────
#[derive(Debug, Serialize)]
pub struct UtilisateurResponse {
    pub id:         Uuid,
    pub email:      String,
    pub nom:        String,
    pub prenom:     Option<String>,
    pub profession: Option<String>,
    pub statut:     String,
    pub created_at: DateTime<Utc>,
}

impl From<Utilisateur> for UtilisateurResponse {
    fn from(u: Utilisateur) -> Self {
        Self {
            id:         u.id,
            email:      u.email,
            nom:        u.nom,
            prenom:     u.prenom,
            profession: u.profession,
            statut:     u.statut,
            created_at: u.created_at,
        }
    }
}

// ── Payload : inscription ────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub email:      String,
    pub password:   String,
    pub nom:        String,
    pub prenom:     Option<String>,
    pub profession: Option<String>,
}

// ── Payload : connexion ──────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email:    String,
    pub password: String,
}

// ── Réponse après login ──────────────────────────────────────
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token:       String,
    pub utilisateur: UtilisateurResponse,
}

// ════════════════════════════════════════════
// CHAMP
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Champ {
    pub id:             Uuid,
    pub utilisateur_id: Uuid,
    pub nom:            String,
    pub description:    Option<String>,
    pub localisation:   Option<String>,
    pub superficie:     Option<f64>,
    pub latitude:       Option<f64>,
    pub longitude:      Option<f64>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ChampPayload {
    pub nom:          String,
    pub description:  Option<String>,
    pub localisation: Option<String>,
    pub superficie:   Option<f64>,
    pub latitude:     Option<f64>,
    pub longitude:    Option<f64>,
}


// ════════════════════════════════════════════
// CULTURE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Culture {
    pub id:                  Uuid,
    pub champ_id:            Uuid,
    pub nom:                 String,
    pub type_culture:        Option<String>,
    pub stade_croissance:    Option<String>,
    pub date_semence:        Option<NaiveDate>,
    pub date_recolte_prevue: Option<NaiveDate>,
    pub notes:               Option<String>,
    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CulturePayload {
    pub nom:                 String,
    pub type_culture:        Option<String>,
    pub stade_croissance:    Option<String>,
    pub date_semence:        Option<NaiveDate>,
    pub date_recolte_prevue: Option<NaiveDate>,
    pub notes:               Option<String>,
}


// ════════════════════════════════════════════
// NOEUD CAPTEUR
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NoeudCapteur {
    pub id:                 Uuid,
    pub nom:                String,
    pub type_capteur:       String,
    pub longitude:          Option<f64>,
    pub latitude:           Option<f64>,
    pub batterie:           Option<i32>,
    pub etat:               String,
    pub surface_couverte:   Option<f64>,
    pub derniere_connexion: Option<DateTime<Utc>>,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NoeudCapteurPayload {
    pub nom:              String,
    pub type_capteur:     String,
    pub longitude:        Option<f64>,
    pub latitude:         Option<f64>,
    pub batterie:         Option<i32>,
    pub surface_couverte: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEtatCapteur {
    pub etat:               String,
    pub batterie:           Option<i32>,
    pub derniere_connexion: Option<DateTime<Utc>>,
}



// ════════════════════════════════════════════
// SEUIL HUMIDITE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SeuilHumidite {
    pub id:               Uuid,
    pub utilisateur_id:   Uuid,
    pub valeur_min:       f64,
    pub valeur_max:       f64,
    pub irrigation_auto:  bool,
    pub type_humidite:    String,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SeuilHumiditePayload {
    pub valeur_min:      f64,
    pub valeur_max:      f64,
    pub irrigation_auto: Option<bool>,
    pub type_humidite:   String,
}   

// ════════════════════════════════════════════
// DONNEE HUMIDITE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DonneeHumidite {
    pub id:               Uuid,
    pub noeud_capteur_id: Uuid,
    pub valeur:           f64,
    pub type_humidite:    String,
    pub date_mesure:      DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DonneeHumiditePayload {
    pub noeud_capteur_id: Uuid,
    pub valeur:           f64,
    pub type_humidite:    Option<String>,
}

// Filtre de période pour l'historique
#[derive(Debug, Deserialize)]
pub struct PeriodeQuery {
    pub debut: Option<DateTime<Utc>>,
    pub fin:   Option<DateTime<Utc>>,
}

// ════════════════════════════════════════════
// IMAGE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Image {
    pub id:               Uuid,
    pub noeud_capteur_id: Uuid,
    pub code:             Option<String>,
    pub longueur:         Option<i32>,
    pub largeur:          Option<i32>,
    pub chemin_stockage:  Option<String>,
    pub taille_octets:    Option<i64>,
    pub format:           Option<String>,
    pub date_capture:     DateTime<Utc>,
    pub est_traitee:      bool,
    pub created_at:       DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ImagePayload {
    pub noeud_capteur_id: Uuid,
    pub code:             Option<String>,
    pub longueur:         Option<i32>,
    pub largeur:          Option<i32>,
    pub chemin_stockage:  Option<String>,
    pub taille_octets:    Option<i64>,
    pub format:           Option<String>,
}

// ════════════════════════════════════════════
// DONNEE TEMPERATURE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DonneeTemperature {
    pub id:               Uuid,
    pub noeud_capteur_id: Uuid,
    pub valeur:           f64,
    pub date_mesure:      DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DonneeTemperaturePayload {
    pub noeud_capteur_id: Uuid,
    pub valeur:           f64,
}

// ════════════════════════════════════════════
// NODE UPLOAD
// ════════════════════════════════════════════
#[derive(Debug, Deserialize)]
pub struct MetricsPayload {
    pub humidity:    Option<f64>,
    pub temperature: Option<f64>,
    pub battery:     Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ModeUpdate {
    pub mode: String,   // "NORMAL" ou "MAINTENANCE"
}

#[derive(Debug, Serialize)]
pub struct ModeResponse {
    pub node_id: String,
    pub mode:    String,
}

// ════════════════════════════════════════════
// DEMANDE CAPTURE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DemandeCaptureDb {
    pub id:             Uuid,
    pub utilisateur_id: Uuid,
    pub node_id:        String,
    pub statut:         String,
    pub image_id:       Option<Uuid>,
    pub message_erreur: Option<String>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DemandeCapture {
    pub id:         Uuid,
    pub node_id:    String,
    pub statut:     String,
    pub image_url:  Option<String>,   // URL directe si terminée
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DemandeCapturePayload {
    pub node_id: String,
}

// ════════════════════════════════════════════
// CHAT IA
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id:             Uuid,
    pub utilisateur_id: Uuid,
    pub titre:          Option<String>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageChat {
    pub id:              Uuid,
    pub conversation_id: Uuid,
    pub role:            String,
    pub contenu:         String,
    pub statut:          String,
    pub image_id:        Option<Uuid>,
    pub created_at:      DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NouveauMessagePayload {
    pub contenu:         String,
    pub conversation_id: Option<Uuid>,  // None = nouvelle conversation
    pub image_id:        Option<Uuid>,  // Image à analyser (optionnel)
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message_user:      MessageChat,
    pub message_assistant: MessageChat,  // statut: en_attente jusqu'à réponse IA
    pub conversation_id:   Uuid,
}

// Ce qu'on envoie au serveur Python
#[derive(Debug, Serialize)]
pub struct IaRequest {
    pub messages:  Vec<IaMessage>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IaMessage {
    pub role:    String,
    pub content: String,
}

// Ce que le serveur Python retourne
#[derive(Debug, Deserialize)]
pub struct IaResponse {
    pub response: String,
}

// ════════════════════════════════════════════
// NOTIFICATION
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id:             Uuid,
    pub utilisateur_id: Uuid,
    pub r#type:         String, // "type" est un mot réservé en Rust
    pub message:        String,
    pub source:         Option<String>,
    pub statut:         String,
    pub date:           DateTime<Utc>,
}

// ════════════════════════════════════════════
// SEUIL TEMPERATURE
// ════════════════════════════════════════════
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SeuilTemperature {
    pub id:             Uuid,
    pub utilisateur_id: Uuid,
    pub valeur_min:     f64,
    pub valeur_max:     f64,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SeuilTemperaturePayload {
    pub valeur_min: f64,
    pub valeur_max: f64,
}