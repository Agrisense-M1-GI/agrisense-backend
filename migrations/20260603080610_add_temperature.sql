CREATE TABLE donnees_temperature (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    noeud_capteur_id UUID NOT NULL REFERENCES noeuds_capteurs(id) ON DELETE CASCADE,
    valeur           FLOAT NOT NULL,          -- en degrés Celsius
    date_mesure      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_donnees_temperature_capteur_date
    ON donnees_temperature(noeud_capteur_id, date_mesure DESC);