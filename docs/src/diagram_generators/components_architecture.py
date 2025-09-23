"""
High Level Architecture diagram generator.
"""

from diagrams import Cluster, Edge
from diagrams.onprem.client import Client
from diagrams.onprem.queue import Kafka
from diagrams.onprem.database import PostgreSQL
from diagrams.onprem.inmemory import Redis
from diagrams.onprem.container import Docker
from diagrams.onprem.monitoring import Grafana, Prometheus
from diagrams.generic.compute import Rack
from diagrams.generic.network import Router, Switch

from base_diagram import BaseDiagram


class ComponentsArchitectureDiagram(BaseDiagram):
    """High Level Architecture diagram implementation."""

    @property
    def name(self) -> str:
        return "Comonents Architecture"

    @property
    def file_name(self) -> str:
        return "components_architecture"

    def generate(self) -> None:
        """Generate the high level architecture diagram."""

        with Cluster("Internet"):
            client = Client("Client")
            server = Rack("Server")

        with Cluster("Doppelganger"):
            comparator = Rack("Behavior Comparator")
            db = PostgreSQL("DB")
            proxy = Router("Proxy")
            grafana = Grafana("Dashboard")
            kafka = Kafka("Kafka")
            prometheus = Prometheus("Prometheus")
            redis = Redis("Cache")
            replicator = Switch("Request Replicator")

        with Cluster("Service"):
            master = Docker("Master")
            with Cluster("Shadows"):
                shadow1 = Docker("Shadow 1")
                shadowx = Docker("Shadow x")

        proxy >> [
            master,
            kafka,
        ]
        proxy << master
        replicator >> Edge() << shadowx
        replicator >> Edge() << shadow1

        shadow1 - Edge(style="dotted") - shadowx

        client >> Edge(label="TCP") >> proxy
        server << Edge(label="TCP") << proxy

        kafka >> replicator

        replicator >> kafka
        replicator >> redis
        comparator >> db
        kafka >> comparator >> prometheus >> grafana


class SidecarRelayArchitectureDiagram(BaseDiagram):
    """Sidecar Relay Architecture diagram with implemented infrastructure."""

    @property
    def name(self) -> str:
        return "Sidecar Relay Architecture"

    @property
    def file_name(self) -> str:
        return "sidecar_relay_architecture"

    def generate(self) -> None:
        """Generate the sidecar relay architecture diagram."""

        with Cluster("Internet"):
            client = Client("Client")

        with Cluster("Kubernetes + Istio"):
            with Cluster("Istio Gateway"):
                istio_gateway = Router("Istio Gateway")

            with Cluster("Primary Service Pod"):
                with Cluster("Istio Sidecar"):
                    envoy_primary = Router("Envoy Proxy")
                primary_service = Docker("Primary Service\n(Rust v1)")
                monitor_logs = Rack("Monitor\n(Async Logging)")

            with Cluster("Shadow Service Pod"):
                with Cluster("Istio Sidecar"):
                    envoy_shadow = Router("Envoy Proxy")
                shadow_service = Docker("Shadow Service\n(Go v2)")
                relay_sidecar = Switch("Relay Sidecar\n(Rust)")

            with Cluster("Monitoring"):
                monitor_service = Rack("Monitor Service\n(Rust)")

            with Cluster("Storage"):
                postgresql = PostgreSQL("PostgreSQL\n(Comparison Results)")
                redis = Redis("Redis\n(Cache)")

            with Cluster("Future Comparison"):
                comparator = Rack("Comparator\n(Offline Analysis)")

        # Client request flow
        client >> Edge(label="HTTP") >> istio_gateway

        # Primary traffic flow (no latency impact)
        istio_gateway >> Edge(label="Primary") >> envoy_primary
        envoy_primary >> primary_service
        primary_service >> envoy_primary
        envoy_primary >> istio_gateway
        istio_gateway >> client

        # Mirror traffic flow (shadow)
        istio_gateway >> Edge(label="Mirror 100%", style="dashed") >> envoy_shadow
        envoy_shadow >> relay_sidecar
        relay_sidecar >> shadow_service
        shadow_service >> relay_sidecar

        # Monitoring (async, no latency impact)
        envoy_primary >> Edge(label="Telemetry", style="dotted") >> monitor_service
        monitor_service >> postgresql

        # Relay logging
        relay_sidecar >> Edge(label="Logs") >> postgresql
        relay_sidecar >> redis

        # Future comparison processing
        postgresql >> Edge(style="dotted") >> comparator
        redis >> Edge(style="dotted") >> comparator
