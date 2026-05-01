CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS audit (
    id uuid NOT NULL,
    user_email character varying(255) DEFAULT now() NOT NULL,
    user_profile character varying(255) NOT NULL,
    method character varying(255) NOT NULL,
    path character varying(255) NOT NULL,
    ip character varying(255) NOT NULL,
    date_of_request date NOT NULL,
    hour_of_request time without time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS average_time_per_doctor (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT average_time_per_doctor_identifier_ifrounidadeid_unique
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS bpa (
    ifrounidadeid integer,
    ifrounidadenome character varying,
    ifrocompetenciaano character varying,
    ifrocompetenciames character varying,
    ifrotabelanome character varying,
    ifrodataatendimento character varying,
    ifropeso character varying,
    ifroaltura character varying,
    ifroprofissionalid character varying,
    ifroprofissionalnome character varying,
    ifroprofissionalcbods character varying,
    ifroprocedimentonome character varying,
    ifroprocedimentosusds character varying,
    ifropacientenome character varying,
    ifropacientedatanascimento character varying,
    ifropacienteidade character varying,
    ifropacientesexods character varying,
    ifropacienteracacords character varying,
    ifropacienteetniads character varying,
    ifropacientenacionalidadeds character varying,
    ifropacientecep character varying,
    ifropacientelogradourocd character varying,
    ifropacientelogradourods character varying,
    ifropacienteendereco character varying,
    ifropacienteendereconumero character varying,
    ifropacientebairro character varying,
    ifrocidcd character varying,
    ifrocompetencia character varying,
    ifrohoraatendimento character varying,
    ifrodiasemana character varying,
    ifroclassificacao character varying,
    ifropacientequeixaprincipal character varying,
    ifropacientelatitude character varying,
    ifropacientelongitude character varying
);

CREATE TABLE IF NOT EXISTS distribuition_of_patients_ages (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT distribuition_of_patients_ages_identifier_ifrounidadeid_unique
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS distribution_of_services_by_hour_group (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT distribution_of_services_by_hour_group_identifier_ifrounidadeid
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS feedbacks (
    id uuid NOT NULL,
    user_name character varying(255) DEFAULT now() NOT NULL,
    feedback character varying(50) NOT NULL,
    prediction_made character varying(255) NOT NULL,
    correct_prediction character varying(255) NOT NULL,
    created_at timestamp without time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS feedbacks_osteoporosis (
    id uuid NOT NULL,
    user_name character varying(255) NOT NULL,
    feedback character varying(50) NOT NULL,
    prediction_made character varying(255) NOT NULL,
    correct_prediction character varying(255) NOT NULL,
    created_at timestamp without time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS feedbacks_tuberculosis (
    id uuid NOT NULL,
    user_name character varying(255) NOT NULL,
    feedback character varying(255) NOT NULL,
    created_at timestamp without time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS forgot_password (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    user_email character varying(255) NOT NULL,
    code_verification integer NOT NULL,
    used boolean NOT NULL,
    created_at timestamp without time zone NOT NULL,
    expiration_at timestamp without time zone
);

CREATE TABLE IF NOT EXISTS heat_map_with_disease_indication (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT heat_map_with_disease_indication_identifier_ifrounidadeid_uniqu
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS heat_map_with_the_number_of_medical_appointments_by_neighborhood (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT heat_map_with_the_number_of_medical_appointments_by_neighborh_i
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS logged (
    id uuid NOT NULL,
    email character varying(255) NOT NULL,
    profile character varying(50),
    ip character varying(50),
    user_agent character varying(255),
    expiration_date timestamp without time zone,
    CONSTRAINT logged_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS map_neighbourhoods (
    bairro character varying,
    map character varying,
    lat character varying,
    long character varying
);

CREATE TABLE IF NOT EXISTS non_doctors (
    ifroprofissionalid bigint,
    ifroprofissionalnome character varying
);

CREATE TABLE IF NOT EXISTS non_nurse (
    ifroprofissionalid bigint,
    ifroprofissionalnome character varying
);

CREATE TABLE IF NOT EXISTS number_of_appointments_per_flow (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_appointments_per_flow_identifier_ifrounidadeid_unique
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS number_of_appointments_per_month (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_appointments_per_month_identifier_ifrounidadeid_uniqu
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS number_of_calls_per_day_of_the_week (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_calls_per_day_of_the_week_identifier_ifrounidadeid_un
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS number_of_visits_per_doctor (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_visits_per_doctor_identifier_ifrounidadeid_unique
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS number_of_visits_per_nurse (
    id uuid DEFAULT public.uuid_generate_v4(),
    identifier text,
    data jsonb,
    ifrounidadeid integer DEFAULT 2 NOT NULL,
    CONSTRAINT number_of_visits_per_nurse_identifier_ifrounidadeid_unique
        UNIQUE (identifier, ifrounidadeid)
);

CREATE TABLE IF NOT EXISTS user_rx (
    id uuid NOT NULL,
    full_name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    profile character varying(50) NOT NULL,
    password character varying(255)
);

CREATE TABLE IF NOT EXISTS users (
    id uuid NOT NULL,
    full_name character varying(255),
    email character varying(255) NOT NULL,
    password character varying(255),
    profile character varying(50),
    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_email_key UNIQUE (email)
);

CREATE TABLE IF NOT EXISTS users_api (
    id uuid NOT NULL,
    full_name character varying(255) DEFAULT now() NOT NULL,
    email character varying(255) NOT NULL,
    profile character varying(255) NOT NULL,
    password character varying(255) NOT NULL,
    allowed_applications text[],
    enabled boolean DEFAULT true NOT NULL,
    allowed_health_units bigint[] DEFAULT ARRAY[]::bigint[]
);
