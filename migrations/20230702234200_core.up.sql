CREATE EXTENSION IF NOT EXISTS "pgcrypto";


CREATE FUNCTION "trigger_set_timestamp"()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TABLE "config" (
	"name" VARCHAR(255) NOT NULL PRIMARY KEY,
	"value" JSONB NOT NULL DEFAULT 'null',
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER "config_set_timestamp" BEFORE UPDATE ON "config" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

INSERT INTO "config" ("name", "value") VALUES
  ('server.address', '"0.0.0.0"'),
  ('server.port', '8080'),
  ('server.uri', '"http://localhost:8080"'),
  ('log_level', '"debug"'),
  ('jwt.secret', (CONCAT('"', translate(encode(gen_random_bytes(255), 'base64'), E'+/=\n', '-_'), '"'))::JSONB),
  ('files.files_folder', '"./files"'),
  ('files.uploads_folder', '"./uploads"');


CREATE FUNCTION config_notify() RETURNS trigger AS $$
DECLARE
  "name" VARCHAR(255);
  "value" JSONB;
BEGIN
  IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
  "name" = NEW."name";
  ELSE
  "name" = OLD."name";
  END IF;
  IF TG_OP != 'UPDATE' OR NEW."value" != OLD."value" THEN
  PERFORM pg_notify('config_channel', json_build_object('table', TG_TABLE_NAME, 'name', "name", 'value', NEW."value", 'action_type', TG_OP)::text);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER "config_notify_update" AFTER UPDATE ON "config" FOR EACH ROW EXECUTE PROCEDURE config_notify();
CREATE TRIGGER "config_notify_insert" AFTER INSERT ON "config" FOR EACH ROW EXECUTE PROCEDURE config_notify();
CREATE TRIGGER "config_notify_delete" AFTER DELETE ON "config" FOR EACH ROW EXECUTE PROCEDURE config_notify();


CREATE TABLE "file" (
  "id" SERIAL PRIMARY KEY,
	"key" VARCHAR(255) NOT NULL,
	"hash" VARCHAR(255) NOT NULL,
  "size" INT4 NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX "file_key_unique_idx" ON "file" ("key");
CREATE TRIGGER "file_set_timestamp" BEFORE UPDATE ON "file" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();
