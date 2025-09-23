#!/bin/bash

# Doppelganger Kubernetes Setup Script

set -e

echo "Setting up Doppelganger on Kubernetes with Istio"

# Check if kubectl is available
if ! command -v kubectl &>/dev/null; then
  echo "kubectl is not installed. Please install kubectl first."
  exit 1
fi

# Check if istioctl is available
if ! command -v istioctl &>/dev/null; then
  echo "istioctl is not installed. Please install Istio first."
  echo "   Download from: https://istio.io/latest/docs/setup/getting-started/"
  exit 1
fi

echo "📋 Checking cluster connectivity..."
kubectl cluster-info >/dev/null 2>&1 || {
  echo "Cannot connect to Kubernetes cluster. Please check your kubeconfig."
  exit 1
}

echo "Installing Istio if not present..."
if ! kubectl get namespace istio-system &>/dev/null; then
  echo "   Installing Istio..."
  istioctl install --set values.defaultRevision=default -y
  kubectl label namespace kube-system istio-injection- --overwrite
else
  echo "   Istio already installed"
fi

echo "Creating namespace and resources..."
kubectl apply -f namespace.yaml

echo "Building and loading Docker images..."
cd ../src
echo "   Building Rust service..."
docker build -t example-service-v1:latest ./example_service_v1/

echo "   Building Go service..."
docker build -t example-service-v2:latest ./example_service_v2/

echo "   Building Rust relay..."
docker build -t relay:latest ./relay/

echo "   Building Rust monitor..."
docker build -t monitor:latest ./monitor/

# Load images into kind cluster if using kind
if kubectl config current-context | grep -q "kind"; then
  echo "   Loading images into kind cluster..."
  kind load docker-image example-service-v1:latest
  kind load docker-image example-service-v2:latest
  kind load docker-image relay:latest
  kind load docker-image monitor:latest
fi

# Load images into kind cluster if using minikube
if kubectl config current-context | grep -q "minikube"; then
  echo "   Loading images into minikube cluster..."
  minikube image load example-service-v1:latest
  minikube image load example-service-v2:latest
  minikube image load relay:latest
  minikube image load monitor:latest
fi

cd ../k8s

echo "Deploying storage components..."
kubectl apply -f storage/

echo "Deploying Kafka..."
kubectl apply -f deployments/kafka.yaml

echo "Deploying services..."
kubectl apply -f deployments/
kubectl apply -f services/

echo "Configuring Istio service mesh..."
kubectl apply -f istio/

echo "Deploying Envoy proxy..."
# Create ConfigMap from external YAML file (cleaner than YAML-in-YAML)
kubectl create configmap envoy-config \
  --from-file=envoy.yaml=envoy/envoy.yaml \
  --namespace=doppelganger \
  --dry-run=client \
  --output=yaml | kubectl apply -f -

kubectl apply -f envoy/envoy-deployment.yaml

echo "Waiting for deployments to be ready..."
kubectl wait --for=condition=available --timeout=60s deployment/postgres -n doppelganger
kubectl wait --for=condition=available --timeout=60s deployment/redis -n doppelganger
kubectl wait --for=condition=available --timeout=60s deployment/zookeeper -n doppelganger
kubectl wait --for=condition=available --timeout=60s deployment/kafka -n doppelganger
kubectl wait --for=condition=available --timeout=60s deployment/example-service-v1 -n doppelganger
kubectl wait --for=condition=available --timeout=60s deployment/example-service-v2 -n doppelganger
kubectl wait --for=condition=available --timeout=60s deployment/monitor-service -n doppelganger
kubectl wait --for=condition=available --timeout=120s deployment/envoy-proxy -n doppelganger

echo "Doppelganger setup complete!"
echo ""
echo "To access the services:"
echo "   Envoy Proxy: kubectl port-forward -n doppelganger svc/envoy-proxy 8080:80"
echo "   Istio Gateway: kubectl port-forward -n istio-system svc/istio-ingressgateway 8081:80"
echo "   Envoy Admin: kubectl port-forward -n doppelganger svc/envoy-proxy 9901:9901"
echo ""
echo "Test endpoints:"
echo "   curl http://localhost:8080/example-endpoint  # Primary + Mirror to relay"
echo "   curl http://localhost:8080/v1               # Primary + Mirror to relay"
echo "   curl http://localhost:8080/v2               # Shadow service via relay"
echo ""
echo "Monitoring:"
echo "   kubectl get pods -n doppelganger"
echo "   kubectl logs -f deployment/envoy-proxy -n doppelganger"
echo "   kubectl logs -f deployment/example-service-v1 -n doppelganger"

