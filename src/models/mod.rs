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
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SeuilHumiditePayload {
    pub valeur_min:      f64,
    pub valeur_max:      f64,
    pub irrigation_auto: Option<bool>,
}   