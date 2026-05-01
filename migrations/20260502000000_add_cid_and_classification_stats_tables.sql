CREATE TABLE IF NOT EXISTS number_of_appointments_per_cid (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_appointments_per_cid_identifier_ifrounidadeid_unique
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS number_of_appointments_per_classification (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_appointments_per_classification_identifier_ifrounidad
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS number_of_medical_appointments_per_classification (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_medical_appointments_per_classification_identifier_un
        UNIQUE (identifier, ifrounidadeid)
);
