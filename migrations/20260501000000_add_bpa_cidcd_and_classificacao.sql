ALTER TABLE bpa
    ADD COLUMN IF NOT EXISTS ifrocidcd character varying(50);

ALTER TABLE bpa
    ADD COLUMN IF NOT EXISTS ifroclassificacao character varying(50);

ALTER TABLE bpa
    ALTER COLUMN ifrocidcd TYPE character varying(50);

ALTER TABLE bpa
    ALTER COLUMN ifroclassificacao TYPE character varying(50);
