CREATE TABLE "files" (
	"id" SERIAL PRIMARY KEY,
	"path" TEXT NOT NULL,
	"type" TEXT,
  "size" BIGINT NOT NULL,
	"updated_at" BIGINT NOT NULL DEFAULT extract(epoch from now() at time zone 'utc'),
	"created_at" BIGINT NOT NULL DEFAULT extract(epoch from now() at time zone 'utc')
);
CREATE UNIQUE INDEX "files_path_unique_idx" ON "files" ("path");