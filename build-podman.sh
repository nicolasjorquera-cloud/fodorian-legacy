#!/bin/bash
# Script para construir y ejecutar el backend como contenedor

# Cargar variables de entorno desde ~/Documents/gcp-c/.env
if [ -f ~/Documents/gcp-c/.env ]; then
    export $(cat ~/Documents/gcp-c/.env | xargs)
fi

echo "Construyendo la imagen del backend..."
podman build -t fodorian-backend .

echo "Ejecutando el contenedor del backend (servicio web en puerto 8080)..."
podman run -d --name fodorian-backend-container \
    -p 8080:8080 \
    -e GOOGLE_PROJECT_ID \
    -e GOOGLE_LOCATION \
    -e GOOGLE_ENGINE_ID \
    fodorian-backend

echo "¡Listo! El backend está corriendo en http://localhost:8080"
echo "Para detener: podman stop fodorian-backend-container && podman rm fodorian-backend-container"
echo "Para detener: podman stop fodorian-backend-container && podman rm fodorian-backend-container"