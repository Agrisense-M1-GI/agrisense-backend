# API Documentation - Agrisense

## 📋 Description générale

**Agrisense** est une API REST développée en **Rust** avec le framework **Axum**. Elle permet de gérer les données agricoles incluant :
- Les utilisateurs et leur authentification
- Les champs agricoles
- Les cultures par champ
- Les capteurs (nœuds capteurs) de monitoring
- Les seuils d'humidité configurables

**Base URL** : `http://localhost:PORT/api`

---

## 🔐 Authentification

### Vue d'ensemble
- L'API utilise des **JWT (JSON Web Tokens)** pour l'authentification
- Les routes publiques : `/api/health`, `/api/auth/register`, `/api/auth/login`
- Toutes les autres routes nécessitent un token JWT valide en header : `Authorization: Bearer <token>`

### Erreurs d'authentification courantes
- `401 Unauthorized` : Token manquant, expiré ou invalide
- `403 Forbidden` : Accès interdit (ressource appartenant à un autre utilisateur)

---

## 📡 Endpoints

### 1️⃣ Health

#### `GET /health`
Vérifie l'état du serveur.

**Authentification** : Non requise

**Réponse (200 OK)**
```json
{
  "status": "ok",
  "service": "agrisense-api",
  "version": "0.1.0"
}
```

**Erreurs possibles** : Aucune

---

### 2️⃣ Authentification

#### `POST /auth/register`
Crée un nouveau compte utilisateur.

**Authentification** : Non requise

**Contenu de la requête**
```json
{
  "email": "utilisateur@example.com",
  "password": "MotDePasse123!",
  "nom": "Dupont",
  "prenom": "Jean",
  "profession": "Agriculteur"
}
```

**Réponse (200 OK)**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "utilisateur": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "utilisateur@example.com",
    "nom": "Dupont",
    "prenom": "Jean",
    "profession": "Agriculteur",
    "statut": "actif",
    "created_at": "2026-05-13T10:00:00Z"
  }
}
```

**Erreurs possibles**
- `400 Bad Request` : Email déjà utilisé ou données invalides
  ```json
  { "error": "Email déjà utilisé" }
  ```

---

#### `POST /auth/login`
Authentifie un utilisateur existant.

**Authentification** : Non requise

**Contenu de la requête**
```json
{
  "email": "utilisateur@example.com",
  "password": "MotDePasse123!"
}
```

**Réponse (200 OK)**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "utilisateur": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "utilisateur@example.com",
    "nom": "Dupont",
    "prenom": "Jean",
    "profession": "Agriculteur",
    "statut": "actif",
    "created_at": "2026-05-13T10:00:00Z"
  }
}
```

**Erreurs possibles**
- `400 Bad Request` : Email ou mot de passe incorrect
  ```json
  { "error": "Email ou mot de passe incorrect" }
  ```

---

### 3️⃣ Utilisateurs

#### `GET /utilisateurs/me`
Récupère les informations de l'utilisateur connecté.

**Authentification** : ✅ Requise

**Headers**
```
Authorization: Bearer <token>
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "utilisateur@example.com",
  "nom": "Dupont",
  "prenom": "Jean",
  "profession": "Agriculteur",
  "statut": "actif",
  "created_at": "2026-05-13T10:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Utilisateur introuvable
  ```json
  { "error": "Utilisateur introuvable" }
  ```

---

#### `PUT /utilisateurs/me`
Met à jour les informations de l'utilisateur connecté.

**Authentification** : ✅ Requise

**Headers**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Contenu de la requête**
```json
{
  "nom": "Dupont",
  "prenom": "Jean",
  "profession": "Agriculteur bio",
  "email": "newemail@example.com"
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "newemail@example.com",
  "nom": "Dupont",
  "prenom": "Jean",
  "profession": "Agriculteur bio",
  "statut": "actif",
  "created_at": "2026-05-13T10:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Utilisateur introuvable

---

### 4️⃣ Champs

#### `GET /champs`
Récupère tous les champs de l'utilisateur connecté.

**Authentification** : ✅ Requise

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
    "nom": "Champ Nord",
    "description": "Champ principal pour le maïs",
    "localisation": "Loire-et-Cher",
    "superficie": 12.5,
    "latitude": 47.5789,
    "longitude": 1.2345,
    "created_at": "2026-05-13T10:00:00Z",
    "updated_at": "2026-05-13T10:00:00Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide

---

#### `GET /champs/:id`
Récupère un champ spécifique de l'utilisateur connecté.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du champ

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
  "nom": "Champ Nord",
  "description": "Champ principal pour le maïs",
  "localisation": "Loire-et-Cher",
  "superficie": 12.5,
  "latitude": 47.5789,
  "longitude": 1.2345,
  "created_at": "2026-05-13T10:00:00Z",
  "updated_at": "2026-05-13T10:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Champ introuvable
  ```json
  { "error": "Champ introuvable" }
  ```

---

#### `POST /champs`
Crée un nouveau champ pour l'utilisateur connecté.

**Authentification** : ✅ Requise

**Contenu de la requête**
```json
{
  "nom": "Champ Est",
  "description": "Nouveau champ pour blé",
  "localisation": "Indre-et-Loire",
  "superficie": 8.75,
  "latitude": 47.4521,
  "longitude": 1.5678
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
  "nom": "Champ Est",
  "description": "Nouveau champ pour blé",
  "localisation": "Indre-et-Loire",
  "superficie": 8.75,
  "latitude": 47.4521,
  "longitude": 1.5678,
  "created_at": "2026-05-13T11:00:00Z",
  "updated_at": "2026-05-13T11:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `400 Bad Request` : Données invalides

---

#### `PUT /champs/:id`
Met à jour un champ existant.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du champ

**Contenu de la requête**
```json
{
  "nom": "Champ Est (Révisé)",
  "description": "Champ pour blé et orge",
  "localisation": "Indre-et-Loire",
  "superficie": 10.0,
  "latitude": 47.4521,
  "longitude": 1.5678
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
  "nom": "Champ Est (Révisé)",
  "description": "Champ pour blé et orge",
  "localisation": "Indre-et-Loire",
  "superficie": 10.0,
  "latitude": 47.4521,
  "longitude": 1.5678,
  "created_at": "2026-05-13T11:00:00Z",
  "updated_at": "2026-05-13T12:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Champ introuvable
- `400 Bad Request` : Données invalides

---

#### `DELETE /champs/:id`
Supprime un champ et toutes ses cultures associées.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du champ

**Réponse (200 OK)**
```json
{
  "message": "Champ supprimé"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Champ introuvable
  ```json
  { "error": "Champ introuvable" }
  ```

---

### 5️⃣ Cultures

#### `GET /champs/:champ_id/cultures`
Récupère toutes les cultures d'un champ.

**Authentification** : ✅ Requise

**Paramètres de route**
- `champ_id` (UUID) : Identifiant du champ

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440010",
    "champ_id": "550e8400-e29b-41d4-a716-446655440001",
    "nom": "Maïs 2026",
    "type_culture": "Maïs",
    "stade_croissance": "Germination",
    "date_semence": "2026-05-01",
    "date_recolte_prevue": "2026-10-15",
    "notes": "Variété productive",
    "created_at": "2026-05-13T10:00:00Z",
    "updated_at": "2026-05-13T10:00:00Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide

---

#### `POST /champs/:champ_id/cultures`
Crée une nouvelle culture dans un champ.

**Authentification** : ✅ Requise

**Paramètres de route**
- `champ_id` (UUID) : Identifiant du champ

**Contenu de la requête**
```json
{
  "nom": "Blé d'hiver 2026",
  "type_culture": "Blé",
  "stade_croissance": "Semis",
  "date_semence": "2025-10-15",
  "date_recolte_prevue": "2026-07-30",
  "notes": "Variété résistante à la sécheresse"
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440011",
  "champ_id": "550e8400-e29b-41d4-a716-446655440001",
  "nom": "Blé d'hiver 2026",
  "type_culture": "Blé",
  "stade_croissance": "Semis",
  "date_semence": "2025-10-15",
  "date_recolte_prevue": "2026-07-30",
  "notes": "Variété résistante à la sécheresse",
  "created_at": "2026-05-13T11:00:00Z",
  "updated_at": "2026-05-13T11:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `400 Bad Request` : Données invalides

---

#### `PUT /champs/:champ_id/cultures/:id`
Met à jour une culture existante.

**Authentification** : ✅ Requise

**Paramètres de route**
- `champ_id` (UUID) : Identifiant du champ
- `id` (UUID) : Identifiant de la culture

**Contenu de la requête**
```json
{
  "nom": "Blé d'hiver 2026",
  "type_culture": "Blé",
  "stade_croissance": "Tallage",
  "date_semence": "2025-10-15",
  "date_recolte_prevue": "2026-07-30",
  "notes": "Variété résistante à la sécheresse - En bonne forme"
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440011",
  "champ_id": "550e8400-e29b-41d4-a716-446655440001",
  "nom": "Blé d'hiver 2026",
  "type_culture": "Blé",
  "stade_croissance": "Tallage",
  "date_semence": "2025-10-15",
  "date_recolte_prevue": "2026-07-30",
  "notes": "Variété résistante à la sécheresse - En bonne forme",
  "created_at": "2026-05-13T11:00:00Z",
  "updated_at": "2026-05-13T12:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Culture introuvable
  ```json
  { "error": "Culture introuvable" }
  ```
- `400 Bad Request` : Données invalides

---

#### `DELETE /champs/:champ_id/cultures/:id`
Supprime une culture.

**Authentification** : ✅ Requise

**Paramètres de route**
- `champ_id` (UUID) : Identifiant du champ
- `id` (UUID) : Identifiant de la culture

**Réponse (200 OK)**
```json
{
  "message": "Culture supprimée"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Culture introuvable
  ```json
  { "error": "Culture introuvable" }
  ```

---

### 6️⃣ Capteurs (Nœuds Capteurs)

#### `GET /capteurs`
Récupère tous les capteurs du système.

**Authentification** : ✅ Requise

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440020",
    "nom": "Capteur Nord-Ouest",
    "type_capteur": "Humidité + Température",
    "longitude": 1.2345,
    "latitude": 47.5789,
    "batterie": 85,
    "etat": "actif",
    "surface_couverte": 2.5,
    "derniere_connexion": "2026-05-13T12:45:00Z",
    "created_at": "2026-05-10T10:00:00Z",
    "updated_at": "2026-05-13T12:45:00Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide

---

#### `GET /capteurs/:id`
Récupère les détails d'un capteur spécifique.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du capteur

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440020",
  "nom": "Capteur Nord-Ouest",
  "type_capteur": "Humidité + Température",
  "longitude": 1.2345,
  "latitude": 47.5789,
  "batterie": 85,
  "etat": "actif",
  "surface_couverte": 2.5,
  "derniere_connexion": "2026-05-13T12:45:00Z",
  "created_at": "2026-05-10T10:00:00Z",
  "updated_at": "2026-05-13T12:45:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```

---

#### `POST /capteurs`
Crée un nouveau capteur (nœud capteur).

**Authentification** : ✅ Requise

**Contenu de la requête**
```json
{
  "nom": "Capteur Sud-Est",
  "type_capteur": "Humidité + Température + Lumière",
  "longitude": 1.3456,
  "latitude": 47.4521,
  "batterie": 100,
  "surface_couverte": 3.0
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440021",
  "nom": "Capteur Sud-Est",
  "type_capteur": "Humidité + Température + Lumière",
  "longitude": 1.3456,
  "latitude": 47.4521,
  "batterie": 100,
  "etat": "actif",
  "surface_couverte": 3.0,
  "derniere_connexion": null,
  "created_at": "2026-05-13T11:00:00Z",
  "updated_at": "2026-05-13T11:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `400 Bad Request` : Données invalides

---

#### `PATCH /capteurs/:id/etat`
Met à jour l'état d'un capteur (appelé par le capteur lui-même).

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du capteur

**Contenu de la requête**
```json
{
  "etat": "actif",
  "batterie": 45,
  "derniere_connexion": "2026-05-13T13:00:00Z"
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440020",
  "nom": "Capteur Nord-Ouest",
  "type_capteur": "Humidité + Température",
  "longitude": 1.2345,
  "latitude": 47.5789,
  "batterie": 45,
  "etat": "actif",
  "surface_couverte": 2.5,
  "derniere_connexion": "2026-05-13T13:00:00Z",
  "created_at": "2026-05-10T10:00:00Z",
  "updated_at": "2026-05-13T13:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```
- `400 Bad Request` : Données invalides

---

### 7️⃣ Seuils d'Humidité

#### `GET /seuils`
Récupère le seuil d'humidité de l'utilisateur connecté.

**Authentification** : ✅ Requise

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440030",
  "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
  "valeur_min": 30.0,
  "valeur_max": 70.0,
  "irrigation_auto": true,
  "created_at": "2026-05-13T10:00:00Z",
  "updated_at": "2026-05-13T10:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Aucun seuil configuré
  ```json
  { "error": "Aucun seuil configuré" }
  ```

---

#### `POST /seuils`
Crée ou remplace le seuil d'humidité de l'utilisateur connecté (UPSERT).

**Authentification** : ✅ Requise

**Contenu de la requête**
```json
{
  "valeur_min": 25.0,
  "valeur_max": 75.0,
  "irrigation_auto": true
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440030",
  "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
  "valeur_min": 25.0,
  "valeur_max": 75.0,
  "irrigation_auto": true,
  "created_at": "2026-05-13T10:00:00Z",
  "updated_at": "2026-05-13T14:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `400 Bad Request` : valeur_min >= valeur_max
  ```json
  { "error": "valeur_min doit être inférieure à valeur_max" }
  ```

---

## 🚨 Messages d'erreurs généraux

### Erreurs HTTP courantes

| Code | Nom | Description |
|------|-----|-------------|
| `400` | Bad Request | Les données envoyées sont invalides |
| `401` | Unauthorized | Authentification requise ou token invalide |
| `403` | Forbidden | Accès interdit (vous ne pouvez accéder qu'à vos propres ressources) |
| `404` | Not Found | Ressource demandée introuvable |
| `500` | Internal Server Error | Erreur interne du serveur |

### Format des erreurs

Toutes les erreurs retournent un JSON avec la clé `error` :

```json
{
  "error": "Description du problème"
}
```

---

## 🔑 Exemple complet d'utilisation

```bash
# 1. Inscription
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123",
    "nom": "Dupont",
    "prenom": "Jean"
  }'

# Réponse contient un token
# TOKEN=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# 2. Récupérer les informations utilisateur
curl -X GET http://localhost:8000/api/utilisateurs/me \
  -H "Authorization: Bearer $TOKEN"

# 3. Créer un champ
curl -X POST http://localhost:8000/api/champs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "nom": "Mon champ",
    "description": "Champ principal",
    "superficie": 10.5,
    "latitude": 47.5789,
    "longitude": 1.2345
  }'

# 4. Créer une culture dans ce champ
CHAMP_ID=550e8400-e29b-41d4-a716-446655440001
curl -X POST http://localhost:8000/api/champs/$CHAMP_ID/cultures \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "nom": "Maïs 2026",
    "type_culture": "Maïs",
    "stade_croissance": "Germination"
  }'

# 5. Créer un capteur
curl -X POST http://localhost:8000/api/capteurs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "nom": "Capteur Principal",
    "type_capteur": "Humidité + Température",
    "latitude": 47.5789,
    "longitude": 1.2345,
    "batterie": 100
  }'

# 6. Configurer les seuils d'humidité
curl -X POST http://localhost:8000/api/seuils \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "valeur_min": 30.0,
    "valeur_max": 70.0,
    "irrigation_auto": true
  }'
```

---

## 📦 Structure des données

### Relations
```
Utilisateur
  ├── Champ (1:N)
  │   └── Culture (1:N)
  └── SeuilHumidite (1:1)

NoeudCapteur (indépendant, global)
```

### Types de données courants
- **UUID** : Identifiant unique (ex: `550e8400-e29b-41d4-a716-446655440000`)
- **DateTime** : Date/heure ISO 8601 UTC (ex: `2026-05-13T10:00:00Z`)
- **NaiveDate** : Date sans heure (ex: `2026-05-13`)
- **Float** : Nombre décimal (ex: `12.5`)
- **Integer** : Nombre entier (ex: `85` pour batterie %)

---

## 📝 Notes supplémentaires

- **Tous les timestamps** sont en UTC (Coordinated Universal Time)
- **Les UUID** sont générés automatiquement par le serveur
- **Les champs nullable** dans les requêtes peuvent être omis (seront NULL en BD)
- **Les mises à jour partielles** sont supportées (utiliser COALESCE côté serveur)
- **CORS** est activé pour accepter les requêtes depuis n'importe quelle origine
- **Isolation des données** : Chaque utilisateur ne peut accéder qu'à ses propres données (champs, cultures, seuils)

