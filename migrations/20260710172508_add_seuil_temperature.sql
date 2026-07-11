-- Add migration script here
CREATE TABLE IF NOT EXISTS seuils_temperature (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    utilisateur_id UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    valeur_min     FLOAT NOT NULL CHECK (valeur_min >= -50),
    valeur_max     FLOAT NOT NULL CHECK (valeur_max <= 100),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT seuil_temp_min_lt_max CHECK (valeur_min < valeur_max),
    CONSTRAINT unique_seuil_temp_utilisateur UNIQUE (utilisateur_id)
);

CREATE OR REPLACE TRIGGER trg_seuils_temperature_updated_at
    BEFORE UPDATE ON seuils_temperature
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
