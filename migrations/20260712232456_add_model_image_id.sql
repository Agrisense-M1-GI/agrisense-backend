ALTER TABLE images
ADD COLUMN model_image_id VARCHAR(100) UNIQUE;

CREATE INDEX idx_images_model_image_id ON images(model_image_id);