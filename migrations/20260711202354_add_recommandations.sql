CREATE TABLE recommandations (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    image_id        UUID NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    etat            VARCHAR(50),     -- Normal, Alerte, Critique
    contenu         TEXT,            -- texte complet retourné par l'IA
    actions         JSONB,           -- liste des actions recommandées
    est_lue         BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recommandations_image ON recommandations(image_id);
CREATE INDEX idx_recommandations_date  ON recommandations(created_at DESC);