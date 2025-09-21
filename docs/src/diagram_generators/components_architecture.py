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
