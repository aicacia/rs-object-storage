CREATE TABLE "objects" (
	"id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	"path" TEXT NOT NULL,
	"type" TEXT,
	"size" INTEGER NOT NULL,
	"updated_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
	"created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
) STRICT;
CREATE UNIQUE INDEX "objects_id_unique_idx" ON "objects" ("id");
CREATE UNIQUE INDEX "objects_path_unique_idx" ON "objects" ("path");
