-- Add migration script here
CREATE TABLE demandes_capture (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    utilisateur_id  UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    node_id         VARCHAR(100) NOT NULL,
    statut          VARCHAR(20) NOT NULL DEFAULT 'en_attente'
                        CHECK (statut IN ('en_attente', 'ack_recu', 'terminee', 'echouee')),
    image_id        UUID REFERENCES images(id) ON DELETE SET NULL,
    message_erreur  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trg_demandes_capture_updated_at
    BEFORE UPDATE ON demandes_capture
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();