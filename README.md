# AgriSense — API Backend

API REST du projet **AgriSense**, une application mobile pour l'agriculture intelligente développée à l'École Nationale Supérieure Polytechnique de Yaoundé (M1 GI — Réseau Mobile et Intelligent).

> ⚠️ Projet académique en cours de développement

---

## À propos du projet

AgriSense permet à un agriculteur de superviser son champ à distance via une application mobile. Le système repose sur un réseau de capteurs sans fil qui collectent des données environnementales (humidité, images) et les transmettent à cette API. Un modèle IA (Indépendant dans un projet Python) analyse les images et génère des recommandations agronomiques.

**Acteurs du système :**
- Principal : L'**agriculteur** — interagit avec l'application mobile
- Secondaire : Le **réseau de capteurs** — nœuds chargé du taux d'humidité et imageurs déployés sur le terrain

**Fonctionnalités principales :**
- Gestion des utilisateurs et authentification JWT
- Supervision du champ (tableau de bord, carte des capteurs)
- Gestion de l'irrigation (manuelle et automatique selon seuil d'humidité)
- Gestion des images capturées par les capteurs
- Réception des recommandations du modèle IA
- Notifications et alertes en temps réel

---

## Architecture globale

```
Application Mobile (Frontend)
          │  HTTP/JSON
          ▼
  API Rust — AgriSense  (:8080)
   │                        │
   │ SQLx                   │ reqwest
   ▼                        ▼
PostgreSQL             Service Python IA
(:5432)                (FastAPI — :8000)
          ▲
          │
  Réseau de capteurs (humidité + imageurs)
```

---

## Stack technique

| Technologie | Rôle |
|---|---|
| **Rust** | Langage principal — performances et sécurité mémoire |
| **Axum 0.7** | Framework web HTTP — routing et handlers |
| **Tokio** | Runtime asynchrone — gestion concurrente des requêtes |
| **Tower / Tower-HTTP** | Middleware — CORS, logs des requêtes |
| **SQLx 0.7** | ORM async pour PostgreSQL — requêtes vérifiées à la compilation |
| **PostgreSQL** | Base de données relationnelle |
| **Argon2** | Hashage sécurisé des mots de passe |
| **jsonwebtoken** | Génération et validation des tokens JWT |
| **reqwest** | Client HTTP pour appeler le service Python IA |
| **Serde / Serde JSON** | Sérialisation / désérialisation JSON |
| **uuid** | Génération des identifiants UUID v4 |
| **chrono** | Gestion des dates et timestamps |
| **tracing** | Logs structurés |
| **thiserror** | Gestion typée des erreurs |
| **validator** | Validation des données entrantes |
| **dotenvy** | Lecture du fichier `.env` |

---

## Prérequis

Avant de commencer, assure-toi d'avoir installé :

- [Rust](https://rustup.rs/) (edition 2021 minimum)
- [PostgreSQL](https://www.postgresql.org/download/) + pgAdmin4 
- [Git](https://git-scm.com/)
- `sqlx-cli` (installé à l'étape 4 ci-dessous)

---

## Installation & démarrage

### 1. Cloner le dépôt

```bash
git clone https://github.com/Agrisense-M1-GI/agrisense-backend.git
cd agrisense
```

### 2. Créer la base de données

Ouvre **pgAdmin4** :
- Clic droit sur **Databases → Create → Database**
- Nom : `agrisense_db`
- Owner : `postgres`
- Clique **Save**

### 3. Créer le fichier `.env`

À la racine du projet, crée un fichier `.env` :

```env
DATABASE_URL=postgres://postgres:TON_MOT_DE_PASSE@localhost:5432/agrisense_db
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
JWT_SECRET=un_secret_long_et_aleatoire
PYTHON_AI_URL=http://localhost:8000
RUST_LOG=info
```

> Remplace `TON_MOT_DE_PASSE` par le mot de passe de ton utilisateur PostgreSQL.

### 4. Installer sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Vérifie l'installation :

```bash
sqlx --version
```

### 5. Appliquer les migrations

```bash
sqlx migrate run
```

Les tables sont créées automatiquement dans `agrisense_db`.
Tu peux vérifier dans pgAdmin4 → `agrisense_db` → **Schemas → Tables**.

> Les migrations sont aussi appliquées automatiquement au démarrage de l'API via `sqlx::migrate!()` dans `main.rs`.

### 6. Compiler le projet

```bash
cargo build
```

La première compilation télécharge toutes les dépendances — elle peut prendre quelques minutes.

### 7. Démarrer l'API

```bash
cargo run
```

Tu devrais voir :

```
🚀 Serveur démarré sur http://127.0.0.1:8080
```

### 8. Vérifier que l'API répond

```bash
curl http://127.0.0.1:8080/health
```

Réponse attendue : `OK`

---

## Structure du projet

```
agrisense/
├── migrations/               # Scripts SQL (appliqués par SQLx)
│   └── XXXX_init_agrisense.sql
├── src/
│   ├── main.rs               # Point d'entrée — démarrage serveur
│   ├── config.rs             # Lecture des variables d'environnement
│   ├── errors.rs             # Gestion globale des erreurs HTTP
│   ├── db/
│   │   └── mod.rs            # Pool de connexion PostgreSQL
│   ├── models/
│   │   └── mod.rs            # Structs mappant les tables (à venir)
│   ├── routes/
│   │   └── mod.rs            # Déclaration de toutes les routes
│   └── middlewares/
│       └── mod.rs            # Middleware JWT (à venir)
├── .env                      # Variables d'environnement (non versionné)
├── .env.example              # Modèle de .env à copier
├── Cargo.toml                # Dépendances Rust
└── README.md
```

---

## Variables d'environnement

| Variable | Description | Exemple |
|---|---|---|
| `DATABASE_URL` | URL de connexion PostgreSQL | `postgres://postgres:pass@localhost:5432/agrisense_db` |
| `SERVER_HOST` | Adresse d'écoute du serveur | `127.0.0.1` |
| `SERVER_PORT` | Port d'écoute | `8080` |
| `JWT_SECRET` | Clé secrète pour signer les tokens JWT | chaîne longue et aléatoire |
| `PYTHON_AI_URL` | URL du service Python hébergeant le modèle IA | `http://localhost:8000` |
| `RUST_LOG` | Niveau de logs | `info` / `debug` |

---

## Roadmap

- [x] Phase 1 — Fondations (Axum, SQLx, AppState, erreurs, logs)
- [x] Phase 2 — Authentification (register, login, JWT)
- [x] Phase 3 — CRUD entités (Utilisateur, Champ, Culture, Capteur)
- [x] Phase 4 — Données capteurs (humidité, images, historiques)
- [ ] Phase 5 — Pipeline IA (analyse image → recommandation)
- [ ] Phase 6 — Notifications
- [ ] Phase 7 — Finalisation (CORS, Swagger, tests)

---

## Équipe

Projet encadré par **Pr. Thomas DJIOTIO** et **M. Juslin KUTCHE**.

| Nom | Matricule |
|---|---|
| MELI NSONWA Yan Evrad | 22p624 |
| NDEFFEU TAMLA Arthur | 22p690 |
| MAGNYE SIMO Cabrelle | 22p643 |
| MENOME FOUKOU Leandre | 22p680 |
| TSOMO TSAGUE Cinthia | 22p649 |
| YACKSON Pascal Dave | 22p692 |
| SAKA NGNITH Wilson | 21p466 |
| MBOUA MBOUA II | 21p473 |