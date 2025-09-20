"""
High Level Architecture diagram generator.
"""

from diagrams import Cluster, Edge
from diagrams.onprem.queue import Kafka
from diagrams.onprem.database import PostgreSQL
from diagrams.onprem.inmemory import Redis
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
        client = Rack("Client")
        server = Rack("Server")

        inbound = Router("Inbound Gateway")
        outbound = Router("Outbound Gateway")
        replicator = Switch("Request Replicator")

        kafka = Kafka("Kafka")
        redis = Redis("Redis")

        # Monitoring side
        collector = Rack("Request Collector")
        db = PostgreSQL("DB")
        comparator = Rack("Behavior Comparator")
        prometheus = Prometheus("Prometheus")
        grafana = Grafana("Grafana")

        # Flow left to right
        client >> Edge(label="TCP") >> inbound >> replicator

        with Cluster("<service>"):
            master = Rack("Master")
            shadow1 = Rack("Shadow 1")
            shadowx = Rack("Shadow x")

            replicator >> master
            master >> outbound
            master >> Edge(style="dotted") >> shadow1
            shadow1 >> Edge(style="dotted") >> shadowx

        outbound >> Edge(label="TCP") >> server
        outbound >> redis

        # Kafka integration
        inbound >> kafka
        replicator >> kafka
        kafka >> collector >> db >> comparator >> prometheus >> grafana
