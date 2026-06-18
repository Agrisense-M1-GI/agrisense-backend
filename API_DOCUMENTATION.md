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

#### `PUT /capteurs/:id`
Met à jour complètement un capteur existant.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du capteur

**Contenu de la requête**
```json
{
  "nom": "Capteur Nord-Ouest (Modifié)",
  "type_capteur": "Humidité + Température + Lumière",
  "longitude": 1.2500,
  "latitude": 47.5850,
  "batterie": 90,
  "surface_couverte": 3.0
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440020",
  "nom": "Capteur Nord-Ouest (Modifié)",
  "type_capteur": "Humidité + Température + Lumière",
  "longitude": 1.2500,
  "latitude": 47.5850,
  "batterie": 90,
  "etat": "actif",
  "surface_couverte": 3.0,
  "derniere_connexion": "2026-05-13T12:45:00Z",
  "created_at": "2026-05-10T10:00:00Z",
  "updated_at": "2026-05-13T15:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```
- `400 Bad Request` : Données invalides

**Notes**
- Cet endpoint met à jour TOUS les champs du capteur (sauf l'ID, created_at, derniere_connexion)
- L'updated_at est automatiquement mis à jour au timestamp actuel

---

#### `DELETE /capteurs/:id`
Supprime un capteur du système.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant du capteur

**Réponse (200 OK)**
```json
{
  "message": "Capteur supprimé avec succès",
  "id": "550e8400-e29b-41d4-a716-446655440020"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```

**Notes**
- La suppression est permanente
- Attention : Toutes les mesures et images associées au capteur peuvent être conservées en base de données
- Les demandes de capture en cours liées à ce capteur peuvent être affectées

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

### 8️⃣ Humidité (Mesures des capteurs)

#### `POST /humidite`
Enregistre une nouvelle mesure d'humidité (appelé par le capteur).

**Authentification** : ❌ Non requise (capteur envoie directement)

**Contenu de la requête**
```json
{
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "valeur": 65.5
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440040",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "valeur": 65.5,
  "date_mesure": "2026-05-13T14:30:00Z"
}
```

**Erreurs possibles**
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```
- `400 Bad Request` : Données invalides

**Notes**
- Si la valeur dépasse les seuils configurés, une **notification d'alerte** est automatiquement créée pour l'utilisateur
- Les seuils sont vérifiés contre `seuils_humidite` dans la base de données

---

#### `GET /humidite/:capteur_id`
Récupère l'historique des mesures d'humidité d'un capteur.

**Authentification** : ✅ Requise

**Paramètres de route**
- `capteur_id` (UUID) : Identifiant du capteur

**Paramètres de requête (optionnels)**
- `debut` (DateTime ISO 8601) : Date/heure de début du filtre (ex: `2026-05-01T00:00:00Z`)
- `fin` (DateTime ISO 8601) : Date/heure de fin du filtre (ex: `2026-05-13T23:59:59Z`)

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440040",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "valeur": 65.5,
    "date_mesure": "2026-05-13T14:30:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440041",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "valeur": 68.2,
    "date_mesure": "2026-05-13T15:00:00Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable

**Exemples d'appels**
```bash
# Toutes les mesures
curl -X GET http://localhost:8000/api/humidite/550e8400-e29b-41d4-a716-446655440020 \
  -H "Authorization: Bearer $TOKEN"

# Mesures du mois de mai 2026
curl -X GET "http://localhost:8000/api/humidite/550e8400-e29b-41d4-a716-446655440020?debut=2026-05-01T00:00:00Z&fin=2026-05-31T23:59:59Z" \
  -H "Authorization: Bearer $TOKEN"
```

**Limite** : Retourne les 500 dernières mesures pour éviter de surcharger la réponse

---

#### `GET /humidite/:capteur_id/derniere`
Récupère la **dernière mesure d'humidité** d'un capteur (la plus récente).

**Authentification** : ✅ Requise

**Paramètres de route**
- `capteur_id` (UUID) : Identifiant du capteur

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440041",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "valeur": 68.2,
  "date_mesure": "2026-05-13T15:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Aucune mesure disponible pour ce capteur
  ```json
  { "error": "Aucune mesure disponible" }
  ```

**Utilité** : Utile pour afficher rapidement l'état actuel d'humidité sans charger tout l'historique

---

### 9️⃣ Images (Captures des capteurs)

#### `POST /images`
Enregistre une nouvelle image capturée par un capteur.

**Authentification** : ❌ Non requise (capteur envoie directement)

**Contenu de la requête**
```json
{
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "code": "IMG_20260513_143000",
  "longueur": 1920,
  "largeur": 1080,
  "chemin_stockage": "/storage/capteurs/550e8400/IMG_20260513_143000.jpg",
  "taille_octets": 2048576,
  "format": "jpg"
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440050",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "code": "IMG_20260513_143000",
  "longueur": 1920,
  "largeur": 1080,
  "chemin_stockage": "/storage/capteurs/550e8400/IMG_20260513_143000.jpg",
  "taille_octets": 2048576,
  "format": "jpg",
  "date_capture": "2026-05-13T14:30:00Z",
  "est_traitee": false,
  "created_at": "2026-05-13T14:30:01Z"
}
```

**Erreurs possibles**
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```
- `400 Bad Request` : Données invalides

**Notes**
- `est_traitee` est `false` par défaut (l'image doit être analysée par le modèle IA)
- Le `chemin_stockage` peut être un chemin absolu ou une URL S3

---

#### `GET /images/:capteur_id`
Récupère l'historique de toutes les images d'un capteur.

**Authentification** : ✅ Requise

**Paramètres de route**
- `capteur_id` (UUID) : Identifiant du capteur

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440050",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "code": "IMG_20260513_143000",
    "longueur": 1920,
    "largeur": 1080,
    "chemin_stockage": "/storage/capteurs/550e8400/IMG_20260513_143000.jpg",
    "taille_octets": 2048576,
    "format": "jpg",
    "date_capture": "2026-05-13T14:30:00Z",
    "est_traitee": true,
    "created_at": "2026-05-13T14:30:01Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable

**Notes** : Les images sont retournées en ordre décroissant par date de capture (les plus récentes en premier)

---

#### `GET /images/:capteur_id/non-traitees`
Récupère les **images non traitées** (pas encore analysées par le modèle IA) d'un capteur.

**Authentification** : ✅ Requise

**Paramètres de route**
- `capteur_id` (UUID) : Identifiant du capteur

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440050",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "code": "IMG_20260513_150000",
    "longueur": 1920,
    "largeur": 1080,
    "chemin_stockage": "/storage/capteurs/550e8400/IMG_20260513_150000.jpg",
    "taille_octets": 2048576,
    "format": "jpg",
    "date_capture": "2026-05-13T15:00:00Z",
    "est_traitee": false,
    "created_at": "2026-05-13T15:00:01Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Aucune image non traitée ou capteur introuvable

**Utilité** : Permet de récupérer la liste des images en attente de traitement pour un pipeline IA/ML

**Notes** : Les images sont retournées en ordre croissant par date de capture (les plus anciennes en premier)

---

#### `GET /images/detail/:id`
Récupère les détails d'une image spécifique.

**Authentification** : ✅ Requise

**Paramètres de route**
- `id` (UUID) : Identifiant de l'image

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440050",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "code": "IMG_20260513_143000",
  "longueur": 1920,
  "largeur": 1080,
  "chemin_stockage": "/storage/capteurs/550e8400/IMG_20260513_143000.jpg",
  "taille_octets": 2048576,
  "format": "jpg",
  "date_capture": "2026-05-13T14:30:00Z",
  "est_traitee": true,
  "created_at": "2026-05-13T14:30:01Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Image introuvable
  ```json
  { "error": "Image introuvable" }
  ```

---

### 🔟 Température (Mesures des capteurs)

#### `POST /temperature`
Enregistre une nouvelle mesure de température (appelé par le capteur).

**Authentification** : ❌ Non requise (capteur envoie directement)

**Contenu de la requête**
```json
{
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "valeur": 22.5
}
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440060",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "valeur": 22.5,
  "date_mesure": "2026-06-04T14:30:00Z"
}
```

**Erreurs possibles**
- `404 Not Found` : Capteur introuvable
  ```json
  { "error": "Capteur introuvable" }
  ```
- `400 Bad Request` : Données invalides

**Notes**
- La température est enregistrée en degrés Celsius
- Chaque mesure est associée à un timestamp automatique

---

#### `GET /temperature/:capteur_id`
Récupère l'historique des mesures de température d'un capteur.

**Authentification** : ✅ Requise

**Paramètres de route**
- `capteur_id` (UUID) : Identifiant du capteur

**Paramètres de requête (optionnels)**
- `debut` (DateTime ISO 8601) : Date/heure de début du filtre (ex: `2026-06-01T00:00:00Z`)
- `fin` (DateTime ISO 8601) : Date/heure de fin du filtre (ex: `2026-06-04T23:59:59Z`)

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440060",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "valeur": 22.5,
    "date_mesure": "2026-06-04T14:30:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440061",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "valeur": 23.1,
    "date_mesure": "2026-06-04T15:00:00Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Capteur introuvable

**Exemples d'appels**
```bash
# Toutes les mesures
curl -X GET http://localhost:8000/api/temperature/550e8400-e29b-41d4-a716-446655440020 \
  -H "Authorization: Bearer $TOKEN"

# Mesures du 4 juin 2026
curl -X GET "http://localhost:8000/api/temperature/550e8400-e29b-41d4-a716-446655440020?debut=2026-06-04T00:00:00Z&fin=2026-06-04T23:59:59Z" \
  -H "Authorization: Bearer $TOKEN"
```

**Limite** : Retourne les 500 dernières mesures pour éviter de surcharger la réponse

---

#### `GET /temperature/:capteur_id/derniere`
Récupère la **dernière mesure de température** d'un capteur (la plus récente).

**Authentification** : ✅ Requise

**Paramètres de route**
- `capteur_id` (UUID) : Identifiant du capteur

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440061",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "valeur": 23.1,
  "date_mesure": "2026-06-04T15:00:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Aucune mesure disponible pour ce capteur
  ```json
  { "error": "Aucune mesure disponible" }
  ```

**Utilité** : Utile pour afficher rapidement la température actuelle sans charger tout l'historique

---

### 1️⃣1️⃣ Demandes de Capture

> ℹ️ **Contexte technique** : La capture d'image implique une communication LoRa entre la station de base (Arduino) et le nœud capteur (Raspberry Pi). Ce processus se déroule en deux phases :
> - **Phase 1 (rapide ~5s)** : La commande `CMD:CAPTURE` est envoyée via LoRa. Le nœud répond avec un ACK.
> - **Phase 2 (variable)** : Le nœud prend la photo, la traite et l'envoie via HTTP. Cela peut prendre plusieurs dizaines de secondes.
>
> C'est pourquoi la capture utilise un **système de polling** : le frontend crée une demande, reçoit un `job_id`, puis interroge régulièrement le statut jusqu'à réception de l'image.

---

#### `POST /capturer`
Crée une nouvelle demande de capture d'image pour un nœud capteur.

**Authentification** : ✅ Requise

**Contenu de la requête**
```json
{
  "node_id": "noode_first_001"
}
```

> ⚠️ `node_id` doit correspondre exactement au champ `nom` du nœud enregistré en base de données.

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440070",
  "node_id": "noode_first_001",
  "statut": "en_attente",
  "image_url": null,
  "created_at": "2026-06-04T15:30:00Z",
  "updated_at": "2026-06-04T15:30:00Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `400 Bad Request` : Une capture est déjà en cours pour ce nœud
  ```json
  { "error": "Une capture est déjà en cours pour ce nœud" }
  ```
- `500 Internal Server Error` : Port série non disponible
  ```json
  { "error": "Port série non disponible sur ce serveur" }
  ```

**Notes**
- Le `statut` initial est `en_attente`
- Il ne peut y avoir qu'une seule capture en cours par nœud
- `image_url` est `null` jusqu'à réception de l'image
- Timeout automatique après **2 minutes** si aucune image n'est reçue

---

#### `GET /capturer/historique`
Récupère l'historique des 50 dernières demandes de capture de l'utilisateur.

**Authentification** : ✅ Requise

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440070",
    "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
    "node_id": "noode_first_001",
    "statut": "terminee",
    "image_id": "550e8400-e29b-41d4-a716-446655440050",
    "message_erreur": null,
    "created_at": "2026-06-04T15:30:00Z",
    "updated_at": "2026-06-04T15:32:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440071",
    "utilisateur_id": "550e8400-e29b-41d4-a716-446655440000",
    "node_id": "noode_first_001",
    "statut": "echouee",
    "image_id": null,
    "message_erreur": "Nœud injoignable — pas de réponse après 3 tentatives",
    "created_at": "2026-06-04T15:35:00Z",
    "updated_at": "2026-06-04T15:35:15Z"
  }
]
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide

**Statuts possibles**
| Statut | Description |
|--------|-------------|
| `en_attente` | Commande envoyée via LoRa, pas encore de ACK |
| `ack_recu` | Le nœud a confirmé la réception, photo en cours |
| `terminee` | Image reçue, stockée et disponible |
| `echouee` | Nœud injoignable, timeout 2min, ou erreur |

---

#### `GET /capturer/:job_id`
Récupère le statut d'une demande de capture. **Utilisé pour le polling.**

**Authentification** : ✅ Requise

**Paramètres de route**
- `job_id` (UUID) : Identifiant retourné par `POST /capturer`

**Réponse — capture en attente**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440070",
  "node_id": "noode_first_001",
  "statut": "en_attente",
  "image_url": null,
  "created_at": "2026-06-04T15:30:00Z",
  "updated_at": "2026-06-04T15:30:00Z"
}
```

**Réponse — ACK reçu (photo en cours)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440070",
  "node_id": "noode_first_001",
  "statut": "ack_recu",
  "image_url": null,
  "created_at": "2026-06-04T15:30:00Z",
  "updated_at": "2026-06-04T15:30:05Z"
}
```

**Réponse — capture terminée**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440070",
  "node_id": "noode_first_001",
  "statut": "terminee",
  "image_url": "http://192.168.1.42:8080/fichiers/noode_first_001/images/noode_first_001_20260604_153000.jpg",
  "created_at": "2026-06-04T15:30:00Z",
  "updated_at": "2026-06-04T15:32:00Z"
}
```

**Réponse — capture échouée**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440070",
  "node_id": "noode_first_001",
  "statut": "echouee",
  "image_url": null,
  "created_at": "2026-06-04T15:30:00Z",
  "updated_at": "2026-06-04T15:30:20Z"
}
```

**Erreurs possibles**
- `401 Unauthorized` : Token absent ou invalide
- `404 Not Found` : Demande introuvable
  ```json
  { "error": "Demande introuvable" }
  ```

---

## 📱 Guide d'intégration Frontend — Capture d'image

Cette section explique comment implémenter la capture d'image côté frontend de bout en bout.

### Flux complet

```
Utilisateur clique "Capturer"
        │
        ▼
POST /api/capturer  →  reçoit { id: jobId, statut: "en_attente" }
        │
        ▼
Afficher spinner "En attente du nœud..."
        │
        ▼
Poll GET /api/capturer/:jobId toutes les 2 secondes
        │
        ├── statut: "en_attente"  →  continuer à attendre
        ├── statut: "ack_recu"    →  afficher "Photo en cours..."
        ├── statut: "terminee"    →  afficher l'image ✅
        └── statut: "echouee"     →  afficher l'erreur ❌
```

---

### Implémentation JavaScript (exemple complet)

```javascript
const API_BASE = 'http://192.168.1.42:8080/api';
const TOKEN    = localStorage.getItem('token');

/**
 * Lance une capture et gère le polling jusqu'à réception de l'image.
 * @param {string} nodeId - Identifiant du nœud (doit correspondre au nom en base)
 * @param {function} onStatut - Callback appelé à chaque changement de statut
 * @param {function} onTerminee - Callback appelé quand l'image est disponible
 * @param {function} onEchouee - Callback appelé en cas d'échec
 */
async function demanderCapture(nodeId, onStatut, onTerminee, onEchouee) {

  // ── Étape 1 : Créer la demande ──────────────────────────────
  let jobId;
  try {
    const res = await fetch(`${API_BASE}/capturer`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${TOKEN}`,
        'Content-Type':  'application/json',
      },
      body: JSON.stringify({ node_id: nodeId }),
    });

    if (!res.ok) {
      const err = await res.json();
      onEchouee(err.error || 'Erreur lors de la demande');
      return;
    }

    const data = await res.json();
    jobId = data.id;
    onStatut('en_attente');

  } catch (e) {
    onEchouee('Impossible de contacter le serveur');
    return;
  }

  // ── Étape 2 : Polling toutes les 2 secondes ─────────────────
  const INTERVALLE_MS  = 2000;
  const TIMEOUT_MAX_MS = 150000;  // 2min30 — légèrement > timeout serveur
  const debut          = Date.now();

  const poll = setInterval(async () => {

    // Sécurité côté client : arrêt si trop long
    if (Date.now() - debut > TIMEOUT_MAX_MS) {
      clearInterval(poll);
      onEchouee('Timeout — aucune réponse du nœud');
      return;
    }

    try {
      const res = await fetch(`${API_BASE}/capturer/${jobId}`, {
        headers: { 'Authorization': `Bearer ${TOKEN}` },
      });

      if (!res.ok) {
        clearInterval(poll);
        onEchouee('Erreur lors de la vérification du statut');
        return;
      }

      const data = await res.json();
      onStatut(data.statut);

      // ── Capture terminée ──────────────────────────────────
      if (data.statut === 'terminee') {
        clearInterval(poll);
        onTerminee(data.image_url);
      }

      // ── Capture échouée ───────────────────────────────────
      if (data.statut === 'echouee') {
        clearInterval(poll);
        onEchouee('Le nœud n\'a pas répondu ou une erreur est survenue');
      }

    } catch (e) {
      // Erreur réseau temporaire — on continue à poller
      console.warn('Erreur réseau temporaire, nouvelle tentative...', e);
    }

  }, INTERVALLE_MS);
}
```

---

### Exemple d'utilisation avec feedback utilisateur

```javascript
// Éléments UI
const btnCapturer  = document.getElementById('btn-capturer');
const statusText   = document.getElementById('status-text');
const imagePreview = document.getElementById('image-preview');
const spinner      = document.getElementById('spinner');

// Messages affichés selon le statut
const MESSAGES_STATUT = {
  'en_attente': '📡 Commande envoyée, attente du nœud...',
  'ack_recu':   '📸 Nœud contacté, prise de photo en cours...',
  'terminee':   '✅ Image reçue !',
  'echouee':    '❌ Échec de la capture',
};

btnCapturer.addEventListener('click', async () => {
  // Désactive le bouton pendant la capture
  btnCapturer.disabled = true;
  spinner.style.display = 'block';
  imagePreview.style.display = 'none';

  await demanderCapture(
    'noode_first_001',

    // onStatut — mis à jour à chaque poll
    (statut) => {
      statusText.textContent = MESSAGES_STATUT[statut] || statut;
    },

    // onTerminee — affiche l'image
    (imageUrl) => {
      spinner.style.display = 'none';
      imagePreview.src = imageUrl;
      imagePreview.style.display = 'block';
      btnCapturer.disabled = false;
    },

    // onEchouee — affiche l'erreur
    (erreur) => {
      spinner.style.display = 'none';
      statusText.textContent = `❌ ${erreur}`;
      btnCapturer.disabled = false;
    }
  );
});
```

---

### Exemple avec React

```jsx
import { useState } from 'react';

const API_BASE = 'http://192.168.1.42:8080/api';

export function BoutonCapture({ nodeId, token }) {
  const [statut,   setStatut]   = useState(null);
  const [imageUrl, setImageUrl] = useState(null);
  const [erreur,   setErreur]   = useState(null);
  const [loading,  setLoading]  = useState(false);

  const MESSAGES = {
    'en_attente': '📡 Attente du nœud...',
    'ack_recu':   '📸 Photo en cours...',
    'terminee':   '✅ Image reçue !',
    'echouee':    '❌ Échec',
  };

  const capturer = async () => {
    setLoading(true);
    setImageUrl(null);
    setErreur(null);

    // 1. Créer la demande
    const res = await fetch(`${API_BASE}/capturer`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type':  'application/json',
      },
      body: JSON.stringify({ node_id: nodeId }),
    });

    if (!res.ok) {
      const err = await res.json();
      setErreur(err.error);
      setLoading(false);
      return;
    }

    const { id: jobId } = await res.json();
    setStatut('en_attente');

    // 2. Polling
    const poll = setInterval(async () => {
      const r    = await fetch(`${API_BASE}/capturer/${jobId}`, {
        headers: { 'Authorization': `Bearer ${token}` },
      });
      const data = await r.json();

      setStatut(data.statut);

      if (data.statut === 'terminee') {
        clearInterval(poll);
        setImageUrl(data.image_url);
        setLoading(false);
      }
      if (data.statut === 'echouee') {
        clearInterval(poll);
        setErreur('Nœud injoignable ou timeout');
        setLoading(false);
      }
    }, 2000);
  };

  return (
    <div>
      <button onClick={capturer} disabled={loading}>
        {loading ? 'Capture en cours...' : '📸 Capturer une image'}
      </button>

      {statut && <p>{MESSAGES[statut]}</p>}
      {erreur  && <p style={{ color: 'red' }}>{erreur}</p>}
      {imageUrl && (
        <img
          src={imageUrl}
          alt="Capture du champ"
          style={{ maxWidth: '100%', marginTop: 16 }}
        />
      )}
    </div>
  );
}
```

---

## 🖼️ Accès aux images stockées

### Comment les images sont stockées

Les images capturées par les nœuds sont stockées sur le serveur dans le dossier :
```
data/nodes/{node_id}/images/{node_id}_{timestamp}.{extension}
```

Exemple :
```
data/nodes/noode_first_001/images/noode_first_001_20260604_153000.jpg
```

### Accéder à une image via HTTP

Le serveur expose directement les images via la route statique `/fichiers` :

```
GET http://{IP_SERVEUR}:{PORT}/fichiers/{node_id}/images/{nom_fichier}
```

Exemple concret :
```
GET http://192.168.1.42:8080/fichiers/noode_first_001/images/noode_first_001_20260604_153000.jpg
```

### Accéder aux details des images via HTTP

Le serveur expose directement les JSON des images via la route statique `/fichiers` :

```
GET http://{IP_SERVEUR}:{PORT}/fichiers/{node_id}/metrics/{nom_fichier}
```

Ici {nom_fichier} correspond au même nom du fichier json de l'image.

Exemple concret :
```
GET http://192.168.1.42:8080/fichiers/noode_first_001/metrics/001_20260604_153000.json
```
Le nom de l'image est : noode_first_001_20260604_153000.jpg

Le nom du capteur est : noode_first_001

Le nom du JSON est    : 20260604_153000.json


**Authentification** : ❌ Non requise — les images et les json sont publiquement accessibles via leur URL.

> ℹ️ L'`image_url` retournée par `GET /capturer/:job_id` est déjà l'URL complète et directement utilisable dans une balise `<img>`.

---

### Récupérer la liste des images d'un capteur

```
GET /api/images/:capteur_id
Authorization: Bearer <token>
```

**Réponse (200 OK)**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440050",
    "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
    "code": "noode_first_001_20260604_153000",
    "chemin_stockage": "data/nodes/noode_first_001/images/noode_first_001_20260604_153000.jpg",
    "taille_octets": 2048576,
    "format": "jpg",
    "date_capture": "2026-06-04T15:30:00Z",
    "est_traitee": false,
    "created_at": "2026-06-04T15:30:00Z"
  }
]
```

Pour construire l'URL d'affichage depuis `chemin_stockage` :
```javascript
// chemin_stockage = "data/nodes/noode_first_001/images/noode_first_001_20260604_153000.jpg"
const imageUrl = `http://192.168.1.42:8080/fichiers/${
  chemin_stockage.replace('data/nodes/', '')
}`;
// → http://192.168.1.42:8080/fichiers/noode_first_001/images/noode_first_001_20260604_153000.jpg
```

---

### Récupérer une image spécifique

```
GET /api/images/detail/:id
Authorization: Bearer <token>
```

Retourne les métadonnées complètes d'une image par son UUID.

---

### Images non encore analysées par l'IA

```
GET /api/images/:capteur_id/non-traitees
Authorization: Bearer <token>
```

Retourne uniquement les images dont `est_traitee = false` — utiles pour savoir quelles images n'ont pas encore été soumises au modèle IA.

---

### 1️⃣2️⃣ Endpoints Nœuds (Communication Pi/Capteurs)

Ces endpoints sont utilisés par les nœuds capteurs (Raspberry Pi) pour communiquer avec le backend. Ils ne nécessitent pas d'authentification puisqu'ils proviennent des appareils eux-mêmes.

#### `GET /node/:node_id/mode`
Le nœud consulte le mode de fonctionnement au démarrage.

**Authentification** : ❌ Non requise

**Paramètres de route**
- `node_id` (String) : Nom/identifiant du nœud

**Réponse (200 OK)**
```json
{
  "node_id": "node_01",
  "mode": "NORMAL"
}
```

**Erreurs possibles**
- `404 Not Found` : Nœud inexistant en base
  ```json
  { "error": "Nœud node_01 inconnu" }
  ```

**Notes**
- Les modes possibles sont : `NORMAL`, `MAINTENANCE`
- Appelé au démarrage du nœud pour récupérer la configuration

---

#### `PUT /node/mode`
Change le mode du système (appelé depuis l'interface admin).

**Authentification** : ❌ Non requise (pour l'instant)

**Contenu de la requête**
```json
{
  "mode": "MAINTENANCE"
}
```

**Réponse (200 OK)**
```json
{
  "node_id": "system",
  "mode": "MAINTENANCE"
}
```

**Erreurs possibles**
- `400 Bad Request` : Mode invalide
  ```json
  { "error": "Mode invalide. Valeurs acceptées : NORMAL, MAINTENANCE" }
  ```

**Notes**
- Tous les nœuds utilisent ce mode global
- Utile pour mettre le système en maintenance sans redémarrer tous les nœuds

---

#### `POST /node/:node_id/upload/image`
Le nœud envoie une image capturée (multipart/form-data).

**Authentification** : ❌ Non requise

**Paramètres de route**
- `node_id` (String) : Nom/identifiant du nœud

**Headers**
```
Content-Type: multipart/form-data
```

**Corps de la requête**
```
file: <fichier image>
```

**Réponse (200 OK)**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440050",
  "noeud_capteur_id": "550e8400-e29b-41d4-a716-446655440020",
  "code": "node_01_20260604_153000",
  "longueur": null,
  "largeur": null,
  "chemin_stockage": "data/nodes/node_01/images/node_01_20260604_153000.jpg",
  "taille_octets": 2048576,
  "format": "jpg",
  "date_capture": null,
  "est_traitee": false,
  "created_at": "2026-06-04T15:30:00Z"
}
```

**Erreurs possibles**
- `404 Not Found` : Nœud inexistant en base
  ```json
  { "error": "Nœud node_01 inconnu" }
  ```
- `400 Bad Request` : Aucun fichier reçu ou erreur multipart
  ```json
  { "error": "Aucun fichier reçu" }
  ```
- `500 Internal Server Error` : Erreur d'écriture disque

**Notes**
- L'image est sauvegardée dans `data/nodes/{node_id}/images/`
- Après réception, ferme automatiquement la demande de capture en cours
- Le fichier est renommé au format : `{node_id}_{timestamp}.{extension}`
- La taille est enregistrée pour le suivi de la bande passante

---

#### `POST /node/:node_id/upload/metrics`
Le nœud envoie un fichier JSON contenant les mesures (humidité, température, batterie).

**Authentification** : ❌ Non requise

**Paramètres de route**
- `node_id` (String) : Nom/identifiant du nœud

**Headers**
```
Content-Type: multipart/form-data
```

**Corps de la requête**
```
file: <fichier JSON avec les métriques>
```

**Format du JSON de métriques**
```json
{
  "humidity": 65.5,
  "temperature": 22.3,
  "battery": 85
}
```

**Réponse (200 OK)**
```json
{
  "status": "ok",
  "node_id": "node_01",
  "enregistre": {
    "humidite": 65.5,
    "temperature": 22.3,
    "batterie": 85
  }
}
```

**Erreurs possibles**
- `404 Not Found` : Nœud inexistant en base
  ```json
  { "error": "Nœud node_01 inconnu" }
  ```
- `400 Bad Request` : JSON invalide ou aucun fichier
  ```json
  { "error": "JSON invalide : ..." }
  ```

**Notes**
- Le fichier brut JSON est sauvegardé dans `data/nodes/{node_id}/metrics/`
- Après parsing et insertion en base :
  - `humidite` → Insérée dans `donnees_humidite` (type: 'air')
  - `temperature` → Insérée dans `donnees_temperature`
  - `batterie` → Met à jour le champ `batterie` du nœud
- Vérifie automatiquement les seuils d'humidité et génère des notifications si dépassement
- Met à jour `derniere_connexion` du nœud

**Statuts des seuils**
- Si humidité < `valeur_min` → Notification d'alerte "Humidité critique (très faible)"
- Si humidité > `valeur_max` → Notification d'alerte "Humidité excessive (très haute)"

---

## 📝 Notes supplémentaires

- **Tous les timestamps** sont en UTC (Coordinated Universal Time)
- **Les UUID** sont générés automatiquement par le serveur
- **Les champs nullable** dans les requêtes peuvent être omis (seront NULL en BD)
- **Les mises à jour partielles** sont supportées (utiliser COALESCE côté serveur)
- **CORS** est activé pour accepter les requêtes depuis n'importe quelle origine
- **Isolation des données** : Chaque utilisateur ne peut accéder qu'à ses propres données (champs, cultures, seuils)
- **Capteurs et mesures** : Les endpoints `/humidite`, `/temperature` et `/images` sont publics pour permettre aux capteurs d'envoyer des données sans authentification
- **Seuils d'humidité** : À chaque nouvelle mesure d'humidité, un contrôle automatique génère des notifications si les seuils sont dépassés