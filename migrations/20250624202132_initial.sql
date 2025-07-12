CREATE TABLE species (
    id SERIAL PRIMARY KEY,
    name TEXT,
    scientific_name TEXT NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    geom geometry(Point, 4326) NOT NULL
);
CREATE UNIQUE INDEX species_unique_idx
ON species (name, scientific_name, observed_at, latitude, longitude);

CREATE INDEX species_geom_gist_idx
ON species
USING GIST (geom);

CREATE TABLE species_translations (
    scientific_name TEXT PRIMARY KEY,
    name_en TEXT,
    name_nl TEXT
);