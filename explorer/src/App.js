// TODO - move to react-table to demonstrate metrics

// Foreign imports
import axios from 'axios';
import {useEffect, useState} from 'react';
// Local imports
import config from './config.js';
import './App.css';

const axios_client = axios.create({baseURL: config.metrics_url})

function App() {

  const [metrics, setMetrics] = useState({});

  useEffect(() => 
    {
      const _interval = setInterval(() => {
        axios_client.get("/metrics").then(response => {
          setMetrics(JSON.parse(response.data));
        });
      }, 250);
    }
  , []);

  // TOOD - stylize the application to make it more dev friendly
  return (
    <div className="App">
      {
      // Check that metrics is not empty
      Object.keys(metrics).length > 0 && 
        <tbody>
          {/* titles */}
          <tr>
              <th>Metric</th>
              {Object.keys(metrics[Object.keys(metrics)[0]]).map((validator) => { return <th> { validator.slice(0,4) + '...' } </th> })}
          </tr>
          {/* values */}
          {Object.keys(metrics).map((metric, _index) => (
              <tr key={metric}>
                  <td>{metric}</td>
                  {Object.keys(metrics[metric]).map((validator)=>{return <td> {metrics[metric][validator]} </td>  })}
              </tr>
          ))}
        </tbody>
    }
    </div>
  );
}

export default App;
