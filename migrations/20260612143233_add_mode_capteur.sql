-- Add migration script here
-- Table pour stocker le mode global du système
CREATE TABLE config_systeme (
    cle    VARCHAR(100) PRIMARY KEY,
    valeur VARCHAR(255) NOT NULL
);

-- Valeur initiale
INSERT INTO config_systeme (cle, valeur) VALUES ('mode', 'NORMAL');