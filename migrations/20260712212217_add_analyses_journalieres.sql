CREATE TABLE analyses_journalieres (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    date_jour   DATE NOT NULL UNIQUE,
    etat        VARCHAR(20),
    contenu     TEXT,
    priorite    VARCHAR(10),
    est_lue     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);