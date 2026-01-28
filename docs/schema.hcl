# Atlas HCL Schema
# Auto-generated from SeaORM entities on 2026-01-28 11:57:15 UTC
# Docs: https://atlasgo.io/atlas-schema/hcl

schema "public" {
  comment = "Zerg application schema"
}

# PostgreSQL Extensions
extension "pgcrypto" {
  schema  = schema.public
  version = "1.3"
}

extension "uuid-ossp" {
  schema  = schema.public
  version = "1.1"
}

table "tasks" {
  schema = schema.public

  column "id" {
    type = sql("uuid")
  }
  column "title" {
    type = sql("varchar(255)")
    null = false
  }
  column "description" {
    type = sql("varchar(255)")
    null = false
  }
  column "completed" {
    type = sql("boolean")
    null = false
  }
  column "project_id" {
    type = sql("uuid")
  }
  column "priority" {
    type = sql("taskpriority")
    null = false
  }
  column "status" {
    type = sql("taskstatus")
    null = false
  }
  column "due_date" {
    type = sql("timestamptz")
  }
  column "created_at" {
    type = sql("timestamptz")
    null = false
  }
  column "updated_at" {
    type = sql("timestamptz")
    null = false
  }

  primary_key {
    columns = [column.id]
  }
}

table "projects" {
  schema = schema.public

  column "id" {
    type = sql("uuid")
  }
  column "name" {
    type = sql("varchar(255)")
    null = false
  }
  column "user_id" {
    type = sql("uuid")
    null = false
  }
  column "description" {
    type = sql("varchar(255)")
    null = false
  }
  column "cloud_provider" {
    type = sql("cloudprovider")
    null = false
  }
  column "region" {
    type = sql("varchar(255)")
    null = false
  }
  column "environment" {
    type = sql("environment")
    null = false
  }
  column "status" {
    type = sql("projectstatus")
    null = false
  }
  column "budget_limit" {
    type = sql("real")
  }
  column "tags" {
    type = sql("jsonb")
    null = false
  }
  column "enabled" {
    type = sql("boolean")
    null = false
  }
  column "created_at" {
    type = sql("timestamptz")
    null = false
  }
  column "updated_at" {
    type = sql("timestamptz")
    null = false
  }

  primary_key {
    columns = [column.id]
  }
}

table "cloud_resources" {
  schema = schema.public

  column "id" {
    type = sql("uuid")
  }
  column "project_id" {
    type = sql("uuid")
    null = false
  }
  column "name" {
    type = sql("varchar(255)")
    null = false
  }
  column "resource_type" {
    type = sql("varchar(255)")
    null = false
  }
  column "status" {
    type = sql("varchar(255)")
    null = false
  }
  column "region" {
    type = sql("varchar(255)")
    null = false
  }
  column "configuration" {
    type = sql("jsonb")
    null = false
  }
  column "cost_per_hour" {
    type = sql("real")
  }
  column "monthly_cost_estimate" {
    type = sql("real")
  }
  column "tags" {
    type = sql("jsonb")
    null = false
  }
  column "enabled" {
    type = sql("boolean")
    null = false
  }
  column "created_at" {
    type = sql("timestamptz")
    null = false
  }
  column "updated_at" {
    type = sql("timestamptz")
    null = false
  }
  column "deleted_at" {
    type = sql("timestamptz")
  }

  primary_key {
    columns = [column.id]
  }

  foreign_key "fk_cloud_resources_entities_project_id" {
    columns     = [column.project_id]
    ref_columns = [table.entities.column.id]
  }
}

