start:
	minikube start
	minikube addons enable ingress
	cd k8s && ./setup.sh

stop:
	minikube stop
	docker stop $$(docker ps -q --filter "ancestor=example-service-v1") 2>/dev/null || true
	docker stop $$(docker ps -q --filter "ancestor=example-service-v2") 2>/dev/null || true
	pkill -f "kubectl port-forward" 2>/dev/null || true

setup:
	cd k8s && ./setup.sh

k9s:
	k9s -n doppelganger

build:
	docker build -t example-service-v1:latest src/example_service_v1
	docker build -t example-service-v2:latest src/example_service_v2

load-images:
	minikube image load example-service-v1:latest
	minikube image load example-service-v2:latest

set-images:
	kubectl set image deployment/example-service-v1 example-service=example-service-v1:latest -n doppelganger
	kubectl set image deployment/example-service-v2 example-service=example-service-v2:latest -n doppelganger

reload:
	make build
	make load-images
	make set-images
	kubectl rollout restart deployment/example-service-v1 -n doppelganger
	kubectl rollout restart deployment/example-service-v2 -n doppelganger

port-forward:
	kubectl port-forward -n doppelganger svc/envoy-proxy 8080:80 &
	kubectl port-forward -n istio-system svc/istio-ingressgateway 8081:80 &
	kubectl port-forward -n doppelganger svc/envoy-proxy 9901:9901 &

clean:
	kubectl delete namespace doppelganger --ignore-not-found
	minikube stop
	minikube delete

