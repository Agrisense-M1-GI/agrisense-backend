-- Ajoute la colonne type à la table existante
ALTER TABLE donnees_humidite
ADD COLUMN type_humidite VARCHAR(10) NOT NULL DEFAULT 'air'
    CHECK (type_humidite IN ('air', 'sol'));