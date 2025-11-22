# Outputs for Auxin Server GCP deployment

output "cloud_run_url" {
  description = "URL of the deployed Cloud Run service"
  value       = google_cloud_run_v2_service.auxin_server.uri
}

output "service_name" {
  description = "Name of the Cloud Run service"
  value       = google_cloud_run_v2_service.auxin_server.name
}

output "service_account_email" {
  description = "Email of the service account used by Cloud Run"
  value       = google_service_account.auxin_server.email
}

output "artifact_registry_repository" {
  description = "Artifact Registry repository for Docker images"
  value       = google_artifact_registry_repository.auxin.name
}

output "artifact_registry_url" {
  description = "URL for pushing Docker images"
  value       = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.auxin.repository_id}"
}

output "storage_bucket_name" {
  description = "Name of the Cloud Storage bucket for Oxen data"
  value       = google_storage_bucket.oxen_data.name
}

output "storage_bucket_url" {
  description = "URL of the Cloud Storage bucket"
  value       = google_storage_bucket.oxen_data.url
}

output "secret_id" {
  description = "ID of the auth token secret in Secret Manager"
  value       = google_secret_manager_secret.auth_token_secret.secret_id
}

output "vpc_connector_name" {
  description = "Name of the VPC connector (if enabled)"
  value       = var.enable_vpc_connector ? google_vpc_access_connector.auxin[0].name : null
}

output "project_id" {
  description = "GCP Project ID"
  value       = var.project_id
}

output "region" {
  description = "GCP region"
  value       = var.region
}

output "environment" {
  description = "Environment name"
  value       = var.environment
}
