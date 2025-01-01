CREATE TABLE "files" (
	"id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	"path" TEXT NOT NULL,
	"type" TEXT,
	"size" INTEGER NOT NULL,
	"updated_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
	"created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
) STRICT;
CREATE UNIQUE INDEX "files_id_unique_idx" ON "files" ("id");
CREATE UNIQUE INDEX "tfiles_path_unique_idx" ON "files" ("path");
