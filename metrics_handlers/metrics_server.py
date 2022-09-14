# To run the metrics serve, run the following below
# export FLASK_APP=metrics_server && flask run 
# the default port is 5000, to allow this in ufw run
# sudo ufw allow 5000
# to check if the port is open run
# sudo ufw status
# be sure to enable the port in the amazon security group

from flask import Flask, send_from_directory
from flask_restful import Api, Resource, reqparse
from flask_cors import CORS # comment this on deployment, CORS allows us to avoid errors on local webserver
from utils import *

app = Flask(__name__)
CORS(app) # comment this on deployment, CORS allows us to avoid errors on local webserver
api = Api(app)

metrics_thresholds_file = open("metrics_thresholds.json")
metrics_thresholds = json.load(metrics_thresholds_file)
gauge_metrics_to_scrape = list(metrics_thresholds["gauges"].keys())
histogram_metrics_to_scrape = list(metrics_thresholds["histograms"].keys())

@app.route("/", defaults={'path':''})
def serve(path):
    return send_from_directory(app.static_folder,'index.html')

class MetricsApiHandler(Resource):
  def get(self):
    return get_metrics(gauge_metrics_to_scrape + histogram_metrics_to_scrape).to_json()

api.add_resource(MetricsApiHandler, '/metrics')