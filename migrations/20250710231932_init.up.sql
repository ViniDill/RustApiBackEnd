CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS clients (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT timezone('brt'::text, now())
);

CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    nickname VARCHAR(255) NOT NULL,
    imei VARCHAR(20) NOT NULL UNIQUE,
    model VARCHAR(100) NOT NULL,
    serial_number VARCHAR(100) NOT NULL, 
    upload_data TIMESTAMP WITH TIME ZONE NOT NULL,
    upload_gps TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT timezone('brt'::text, now())
);
