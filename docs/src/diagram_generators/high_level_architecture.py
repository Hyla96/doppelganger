"""
High Level Architecture diagram generator.
"""

from diagrams import Cluster, Edge
from diagrams.onprem.analytics import Spark
from diagrams.onprem.compute import Server
from diagrams.onprem.database import PostgreSQL
from diagrams.onprem.inmemory import Redis
from diagrams.onprem.aggregator import Fluentd
from diagrams.onprem.monitoring import Grafana, Prometheus
from diagrams.onprem.network import Nginx
from diagrams.onprem.queue import Kafka

from base_diagram import BaseDiagram


class HighLevelArchitectureDiagram(BaseDiagram):
    """High Level Architecture diagram implementation."""

    @property
    def name(self) -> str:
        return "Advanced Web Service with On-Premises (colored)"

    @property
    def file_name(self) -> str:
        return "advanced_web_service"

    def generate(self) -> None:
        """Generate the high level architecture diagram."""
        ingress = Nginx("ingress")

        metrics = Prometheus("metric")
        metrics << Edge(color="firebrick", style="dashed") << Grafana("monitoring")

        with Cluster("Service Cluster"):
            grpcsvc = [Server("grpc1"), Server("grpc2"), Server("grpc3")]

        with Cluster("Sessions HA"):
            primary = Redis("session")
            (
                primary - Edge(color="brown", style="dashed") - Redis("replica")
                << Edge(label="collect")
                << metrics
            )
            grpcsvc >> Edge(color="brown") >> primary

        with Cluster("Database HA"):
            primary = PostgreSQL("users")
            (
                primary - Edge(color="brown", style="dotted") - PostgreSQL("replica")
                << Edge(label="collect")
                << metrics
            )
            grpcsvc >> Edge(color="black") >> primary

        aggregator = Fluentd("logging")
        (
            aggregator
            >> Edge(label="parse")
            >> Kafka("stream")
            >> Edge(color="black", style="bold")
            >> Spark("analytics")
        )

        (
            ingress
            >> Edge(color="darkgreen")
            << grpcsvc
            >> Edge(color="darkorange")
            >> aggregator
        )
