-- Suppression de l'ancienne contrainte d'unicité basée uniquement sur l'utilisateur
ALTER TABLE seuils_humidite DROP CONSTRAINT unique_seuil_utilisateur;

-- Ajout de la colonne type_humidite
ALTER TABLE seuils_humidite 
ADD COLUMN type_humidite VARCHAR(10) NOT NULL DEFAULT 'sol'
CHECK (type_humidite IN ('air', 'sol'));

-- Nouvelle contrainte d'unicité sur l'association utilisateur et type
ALTER TABLE seuils_humidite 
ADD CONSTRAINT unique_seuil_utilisateur_type UNIQUE (utilisateur_id, type_humidite);
