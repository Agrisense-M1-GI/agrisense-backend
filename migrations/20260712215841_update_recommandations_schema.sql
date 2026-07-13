-- Aligne avec le vrai format du callback du service IA
ALTER TABLE recommandations
    ADD COLUMN IF NOT EXISTS sensor_id VARCHAR(100),
    ADD COLUMN IF NOT EXISTS priorite  VARCHAR(10),
    ADD COLUMN IF NOT EXISTS actions_texte TEXT;