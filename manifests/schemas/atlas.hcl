# Atlas Project Configuration
# Docs: https://atlasgo.io/atlas-schema/projects
#
# Using versioned migrations (SQL files are source of truth)
# Migrations are in: manifests/migrations/mydatabase/

variable "database_url" {
  type    = string
  default = getenv("DATABASE_URL")
}

# =============================================================================
# Environments
# =============================================================================

# Local development (docker-compose, port 5432)
env "local" {
  url = "postgres://myuser:mypassword@localhost:5432/mydatabase?sslmode=disable"
  dev = "docker://postgres/18/dev?search_path=public"
  migration {
    dir = "file://manifests/migrations/mydatabase"
  }
}

# Kind cluster (kompose, port 5433)
env "cluster" {
  url = "postgres://myuser:mypassword@localhost:5433/mydatabase?sslmode=disable"
  dev = "docker://postgres/18/dev?search_path=public"
  migration {
    dir = "file://manifests/migrations/mydatabase"
  }
}

# CNPG (production-like)
env "cnpg" {
  url = var.database_url
  migration {
    dir = "file://manifests/migrations/mydatabase"
  }
}

# Production
env "prod" {
  url = var.database_url
  migration {
    dir = "file://manifests/migrations/mydatabase"
  }
}

# =============================================================================
# Lint Rules
# =============================================================================

lint {
  destructive {
    error = true
  }
  data_depend {
    error = true
  }
}

# =============================================================================
# Diff Settings
# =============================================================================

diff {
  skip {
    drop_schema = true
    drop_table  = true
  }
}
