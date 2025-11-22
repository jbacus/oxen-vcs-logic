# Auxin Server - Google Cloud Platform Infrastructure
# Terraform configuration for deploying auxin-server to GCP

terraform {
  required_version = ">= 1.5"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 5.0"
    }
  }

  # Optional: Configure remote state storage
  # backend "gcs" {
  #   bucket = "auxin-terraform-state"
  #   prefix = "terraform/state"
  # }
}

# Configure the Google Cloud Provider
provider "google" {
  project = var.project_id
  region  = var.region
}

provider "google-beta" {
  project = var.project_id
  region  = var.region
}

# Enable required APIs
resource "google_project_service" "services" {
  for_each = toset([
    "run.googleapis.com",
    "artifactregistry.googleapis.com",
    "cloudbuild.googleapis.com",
    "secretmanager.googleapis.com",
    "storage.googleapis.com",
    "vpcaccess.googleapis.com",
    "compute.googleapis.com",
  ])

  service            = each.value
  disable_on_destroy = false
}

# Artifact Registry for Docker images
resource "google_artifact_registry_repository" "auxin" {
  location      = var.region
  repository_id = "auxin"
  description   = "Docker repository for Auxin Server images"
  format        = "DOCKER"

  depends_on = [google_project_service.services]
}

# Cloud Storage bucket for Oxen data persistence
resource "google_storage_bucket" "oxen_data" {
  name          = "${var.project_id}-auxin-oxen-data"
  location      = var.region
  force_destroy = var.environment == "dev" # Only allow force destroy in dev

  uniform_bucket_level_access = true

  versioning {
    enabled = var.enable_versioning
  }

  lifecycle_rule {
    condition {
      age = 90
    }
    action {
      type = "Delete"
    }
  }

  depends_on = [google_project_service.services]
}

# Secret Manager for sensitive configuration
resource "google_secret_manager_secret" "auth_token_secret" {
  secret_id = "auxin-auth-secret"

  replication {
    auto {}
  }

  depends_on = [google_project_service.services]
}

# Secret version (you'll need to populate this with actual secret value)
resource "google_secret_manager_secret_version" "auth_token_secret_version" {
  secret      = google_secret_manager_secret.auth_token_secret.id
  secret_data = var.auth_token_secret # Set via terraform.tfvars or environment
}

# Service account for Cloud Run
resource "google_service_account" "auxin_server" {
  account_id   = "auxin-server"
  display_name = "Auxin Server Service Account"
  description  = "Service account for auxin-server Cloud Run service"
}

# Grant Cloud Run service account access to secrets
resource "google_secret_manager_secret_iam_member" "auxin_server_secret_access" {
  secret_id = google_secret_manager_secret.auth_token_secret.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.auxin_server.email}"
}

# Grant Cloud Run service account access to storage bucket
resource "google_storage_bucket_iam_member" "auxin_server_storage_access" {
  bucket = google_storage_bucket.oxen_data.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.auxin_server.email}"
}

# VPC Connector for Cloud Run (if private services needed)
resource "google_vpc_access_connector" "auxin" {
  count = var.enable_vpc_connector ? 1 : 0

  name          = "auxin-vpc-connector"
  region        = var.region
  ip_cidr_range = "10.8.0.0/28"
  network       = "default"

  depends_on = [google_project_service.services]
}

# Cloud Run service
resource "google_cloud_run_v2_service" "auxin_server" {
  name     = "auxin-server"
  location = var.region

  template {
    service_account = google_service_account.auxin_server.email

    scaling {
      min_instance_count = var.min_instances
      max_instance_count = var.max_instances
    }

    containers {
      image = "${var.region}-docker.pkg.dev/${var.project_id}/auxin/auxin-server:latest"

      ports {
        container_port = 3000
      }

      resources {
        limits = {
          cpu    = var.cpu
          memory = var.memory
        }
      }

      env {
        name  = "SYNC_DIR"
        value = "/var/oxen/data"
      }

      env {
        name  = "OXEN_SERVER_PORT"
        value = "3000"
      }

      env {
        name  = "OXEN_SERVER_HOST"
        value = "0.0.0.0"
      }

      env {
        name  = "RUST_LOG"
        value = var.rust_log_level
      }

      env {
        name  = "GCS_BUCKET"
        value = google_storage_bucket.oxen_data.name
      }

      env {
        name = "AUTH_TOKEN_SECRET"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.auth_token_secret.secret_id
            version = "latest"
          }
        }
      }

      # Optional: Mount GCS bucket via Cloud Storage FUSE
      # volume_mounts {
      #   name       = "oxen-data"
      #   mount_path = "/var/oxen/data"
      # }
    }

    # Optional: GCS volume
    # volumes {
    #   name = "oxen-data"
    #   gcs {
    #     bucket    = google_storage_bucket.oxen_data.name
    #     read_only = false
    #   }
    # }

    # VPC connector (if enabled)
    dynamic "vpc_access" {
      for_each = var.enable_vpc_connector ? [1] : []
      content {
        connector = google_vpc_access_connector.auxin[0].id
        egress    = "ALL_TRAFFIC"
      }
    }
  }

  traffic {
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
    percent = 100
  }

  depends_on = [
    google_project_service.services,
    google_artifact_registry_repository.auxin,
  ]
}

# Cloud Run IAM - Allow public access (adjust as needed)
resource "google_cloud_run_service_iam_member" "public_access" {
  count = var.allow_public_access ? 1 : 0

  location = google_cloud_run_v2_service.auxin_server.location
  service  = google_cloud_run_v2_service.auxin_server.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# Cloud Run IAM - Specific users/service accounts (if not public)
resource "google_cloud_run_service_iam_member" "authorized_invokers" {
  for_each = var.allow_public_access ? [] : toset(var.authorized_invokers)

  location = google_cloud_run_v2_service.auxin_server.location
  service  = google_cloud_run_v2_service.auxin_server.name
  role     = "roles/run.invoker"
  member   = each.value
}

# Cloud Build trigger (optional - for automatic deployments)
resource "google_cloudbuild_trigger" "auxin_server_deploy" {
  count = var.enable_auto_deploy ? 1 : 0

  name        = "auxin-server-deploy"
  description = "Trigger for deploying auxin-server on push to main"

  github {
    owner = var.github_owner
    name  = var.github_repo

    push {
      branch = "^main$"
    }
  }

  filename = "auxin-server/gcp/cloudbuild.yaml"

  substitutions = {
    _REGION                  = var.region
    _ARTIFACT_REGISTRY_REPO  = "auxin"
    _SERVICE_NAME            = "auxin-server"
    _MEMORY                  = var.memory
    _CPU                     = var.cpu
    _MIN_INSTANCES           = tostring(var.min_instances)
    _MAX_INSTANCES           = tostring(var.max_instances)
    _RUST_LOG                = var.rust_log_level
    _VPC_CONNECTOR           = var.enable_vpc_connector ? google_vpc_access_connector.auxin[0].name : ""
  }

  depends_on = [google_project_service.services]
}

# Grant Cloud Build permission to deploy to Cloud Run
resource "google_project_iam_member" "cloudbuild_run_admin" {
  count = var.enable_auto_deploy ? 1 : 0

  project = var.project_id
  role    = "roles/run.admin"
  member  = "serviceAccount:${data.google_project.project.number}@cloudbuild.gserviceaccount.com"
}

resource "google_project_iam_member" "cloudbuild_sa_user" {
  count = var.enable_auto_deploy ? 1 : 0

  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${data.google_project.project.number}@cloudbuild.gserviceaccount.com"
}

# Data source for project information
data "google_project" "project" {
  project_id = var.project_id
}
