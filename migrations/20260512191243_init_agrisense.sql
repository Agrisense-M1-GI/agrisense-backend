-- ============================================================
--  AgriSense — Migration initiale
--  À placer dans : migrations/XXXXXXXXXXXXXX_init_agrisense.sql
-- ============================================================

-- Extension pour les UUID
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- ============================================================
-- TABLE : utilisateurs
-- ============================================================
CREATE TABLE utilisateurs (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,               -- mot de passe hashé (bcrypt/argon2)
    nom           VARCHAR(100) NOT NULL,
    prenom        VARCHAR(100),
    profession    VARCHAR(150),
    statut        VARCHAR(20) NOT NULL DEFAULT 'actif'
                      CHECK (statut IN ('actif', 'inactif', 'suspendu')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- TABLE : champs
-- ============================================================
CREATE TABLE champs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    utilisateur_id  UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    nom             VARCHAR(150) NOT NULL,
    description     TEXT,
    localisation    VARCHAR(255),                      -- adresse ou description textuelle
    superficie      FLOAT,                             -- en hectares
    latitude        DOUBLE PRECISION,                  -- coordonnées GPS
    longitude       DOUBLE PRECISION,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- TABLE : cultures  (composition avec champ)
-- ============================================================
CREATE TABLE cultures (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    champ_id         UUID NOT NULL REFERENCES champs(id) ON DELETE CASCADE,
    nom              VARCHAR(150) NOT NULL,
    type             VARCHAR(100),                     -- céréale, légume, fruit...
    stade_croissance VARCHAR(100),                     -- semis, croissance, floraison, récolte
    date_semence     DATE,
    date_recolte_prevue DATE,                          -- ajout pertinent pour le suivi
    notes            TEXT,                             -- observations libres
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- TABLE : noeuds_capteurs
-- ============================================================
CREATE TABLE noeuds_capteurs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nom         VARCHAR(150) NOT NULL,
    type        VARCHAR(50) NOT NULL
                    CHECK (type IN ('humidite', 'imageur', 'mixte')),
    longitude   DOUBLE PRECISION,
    latitude    DOUBLE PRECISION,
    batterie    INTEGER CHECK (batterie BETWEEN 0 AND 100), -- pourcentage
    etat        VARCHAR(20) NOT NULL DEFAULT 'actif'
                    CHECK (etat IN ('actif', 'inactif', 'erreur')),
    surface_couverte FLOAT,                            -- surface couverte en m²
    derniere_connexion TIMESTAMPTZ,                    -- pour surveiller les déconnexions
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- ============================================================
-- TABLE : donnees_humidite
-- ============================================================
CREATE TABLE donnees_humidite (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    noeud_capteur_id UUID NOT NULL REFERENCES noeuds_capteurs(id) ON DELETE CASCADE,
    valeur          FLOAT NOT NULL CHECK (valeur BETWEEN 0 AND 100), -- pourcentage d'humidité
    date_mesure     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Index partiel pour requêtes fréquentes sur les données récentes
    CONSTRAINT valeur_humidite_valide CHECK (valeur >= 0)
);

-- Index pour accélérer les requêtes historiques par capteur
CREATE INDEX idx_donnees_humidite_capteur_date
    ON donnees_humidite(noeud_capteur_id, date_mesure DESC);


-- ============================================================
-- TABLE : images
-- ============================================================
CREATE TABLE images (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    noeud_capteur_id UUID NOT NULL REFERENCES noeuds_capteurs(id) ON DELETE CASCADE,
    code             VARCHAR(100) UNIQUE,               -- identifiant métier de l'image
    longueur         INTEGER,                           -- en pixels
    largeur          INTEGER,                           -- en pixels
    chemin_stockage  VARCHAR(500),                      -- chemin ou URL de l'image stockée
    taille_octets    BIGINT,                            -- taille du fichier
    format           VARCHAR(10) DEFAULT 'jpg'
                         CHECK (format IN ('jpg', 'png', 'webp')),
    date_capture     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    est_traitee      BOOLEAN NOT NULL DEFAULT FALSE,    -- a-t-elle été analysée par l'IA ?
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_images_capteur ON images(noeud_capteur_id, date_capture DESC);


-- ============================================================
-- TABLE : recommandations
-- (produit par le ModèleIA externe — on stocke le résultat)
-- ============================================================
CREATE TABLE recommandations (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    image_id    UUID NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    type        VARCHAR(100) NOT NULL,                 -- ex: 'traitement', 'irrigation', 'récolte'
    description TEXT NOT NULL,
    niveau_confiance FLOAT CHECK (niveau_confiance BETWEEN 0 AND 1), -- score retourné par l'IA
    date         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    est_lue      BOOLEAN NOT NULL DEFAULT FALSE        -- l'agriculteur a-t-il consulté ?
);


-- ============================================================
-- TABLE : irrigations
-- ============================================================
CREATE TABLE irrigations (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    utilisateur_id   UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE SET NULL,
    noeud_capteur_id UUID NOT NULL REFERENCES noeuds_capteurs(id) ON DELETE SET NULL,
    quant_eau        FLOAT NOT NULL CHECK (quant_eau > 0),  -- en litres
    temps_arrosage   INTEGER NOT NULL CHECK (temps_arrosage > 0), -- en secondes
    mode             VARCHAR(20) NOT NULL DEFAULT 'manuel'
                         CHECK (mode IN ('manuel', 'automatique')),
    date             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    statut           VARCHAR(20) NOT NULL DEFAULT 'effectuee'
                         CHECK (statut IN ('effectuee', 'echouee', 'en_cours'))
);

CREATE INDEX idx_irrigations_utilisateur ON irrigations(utilisateur_id, date DESC);


-- ============================================================
-- TABLE : notifications
-- ============================================================
CREATE TABLE notifications (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    utilisateur_id UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    type           VARCHAR(50) NOT NULL
                       CHECK (type IN ('alerte_critique', 'avertissement', 'info', 'recommandation')),
    message        TEXT NOT NULL,
    source         VARCHAR(50),                        -- 'irrigation', 'humidite', 'image'...
    statut         VARCHAR(20) NOT NULL DEFAULT 'non_lue'
                       CHECK (statut IN ('lue', 'non_lue')),
    date           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_utilisateur ON notifications(utilisateur_id, date DESC);


-- ============================================================
-- TABLE : seuils_humidite
-- (config par utilisateur — pour le déclenchement automatique)
-- ============================================================
CREATE TABLE seuils_humidite (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    utilisateur_id UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    valeur_min     FLOAT NOT NULL CHECK (valeur_min >= 0),   -- seuil bas → déclenche irrigation
    valeur_max     FLOAT NOT NULL CHECK (valeur_max <= 100), -- seuil haut → alerte excès d'eau
    irrigation_auto BOOLEAN NOT NULL DEFAULT FALSE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT seuil_min_lt_max CHECK (valeur_min < valeur_max)
);


-- ============================================================
-- Trigger : mise à jour automatique de updated_at
-- ============================================================
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_utilisateurs_updated_at
    BEFORE UPDATE ON utilisateurs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_champs_updated_at
    BEFORE UPDATE ON champs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_cultures_updated_at
    BEFORE UPDATE ON cultures
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_capteurs_updated_at
    BEFORE UPDATE ON noeuds_capteurs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_seuils_updated_at
    BEFORE UPDATE ON seuils_humidite
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();