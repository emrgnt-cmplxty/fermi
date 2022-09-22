import { Box, Card, Divider, Tab, Tabs, Typography } from '@mui/material'
import { DisplayBox } from 'components/DisplayBox/DisplayBox'
import Highcharts from 'highcharts/highstock'
import HighchartsReact from 'highcharts-react-official'
import { useEffect, useRef, useState } from 'react'
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  Pie,
  PieChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { render } from 'sass'

import styles from './PLDashboard.module.scss'
import { PositionsTable } from './PositionsTable'
import { WalletTable } from './WalletTable'

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042']

interface DataCell {
  date: string
  pnl: number
}

const CumulativeLineChart = ({
  data,
  width,
  height,
}: {
  data: DataCell[]
  width: number
  height: number
}) => {
  return (
    <Card className={styles.overview}>
      <Box className={styles.title}>
        <Typography>Cumulative PNL(%) 30</Typography>
      </Box>
      <LineChart
        data={data}
        width={width}
        height={height}
        margin={{
          top: 30,
          right: 70,
          left: 20,
          bottom: 5,
        }}
      >
        <XAxis dataKey="date" />
        <YAxis
          tickFormatter={(tick) => {
            return `${tick}%`
          }}
          tickCount={3}
        />
        <Tooltip />
        <CartesianGrid vertical={false} />
        <Line type="monotone" dataKey="pnl" stroke="#74FBE6" dot={false} />
      </LineChart>
    </Card>
  )
}

const DailyBarChart = ({
  data,
  width,
  height,
}: {
  data: DataCell[]
  width: number
  height: number
}) => {
  return (
    <Card className={styles.overview}>
      <Box className={styles.title}>
        <Typography>Daily PNL</Typography>
      </Box>
      <BarChart
        data={data}
        width={width}
        height={height}
        margin={{
          top: 30,
          right: 70,
          left: 20,
          bottom: 5,
        }}
      >
        <XAxis dataKey="date" />
        <YAxis
          tickFormatter={(tick) => {
            return `${tick}%`
          }}
          tickCount={4}
        />
        <Tooltip cursor={{ fill: 'transparent' }} />
        <CartesianGrid vertical={false} />
        <ReferenceLine y={0} stroke="#FFFFFF" />
        <Bar dataKey="pnl">
          {data.map((entry, index) => (
            <Cell key={index} fill={entry.pnl > 0 ? '#22dd8f' : 'red'} />
          ))}
        </Bar>
      </BarChart>
    </Card>
  )
}

interface AllocCell {
  name: string
  amount: number
}

const AllocPieChart = ({
  data,
  width,
  height,
}: {
  data: AllocCell[]
  width: number
  height: number
}) => {
  const renderLabel = function (entry: AllocCell) {
    return entry.name
  }
  return (
    <Card className={styles.overview}>
      <Box className={styles.title}>
        <Typography>Asset Allocation</Typography>
      </Box>
      <PieChart width={width} height={height}>
        <Pie
          nameKey="name"
          dataKey="amount"
          data={data}
          innerRadius={40}
          outerRadius={80}
          fill="#82ca9d"
          label={renderLabel}
        >
          {data.map((entry, index) => (
            <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
          ))}
        </Pie>
        <Tooltip />
      </PieChart>
    </Card>
  )
}

export const PLDashboard = () => {
  const [tab, setTab] = useState('Positions')
  const [graphDim1, setGraphDim1] = useState({ width: 0, height: 0 })
  const [graphDim2, setGraphDim2] = useState({ width: 0, height: 0 })

  const graphRef1 = useRef<HTMLDivElement>(null)
  const graphRef2 = useRef<HTMLDivElement>(null)

  const data = [
    {
      date: '6-1',

      pnl: 15,
    },
    {
      date: '6-2',

      pnl: 20,
    },
    {
      date: '6-3',

      pnl: 0,
    },
    {
      date: '6-4',

      pnl: -10,
    },
    {
      date: '6-5',

      pnl: -20,
    },
    {
      date: '6-6',

      pnl: 30,
    },
    {
      date: '6-7',

      pnl: 50,
    },
  ]

  const data2 = [
    {
      name: 'BTC',
      amount: 65.01,
    },
    {
      name: 'ETH',
      amount: 10.22,
    },
    {
      name: 'LINK',
      amount: 15.89,
    },
    {
      name: 'XLM',
      amount: 5.88,
    },
    {
      name: 'COMP',
      amount: 3,
    },
  ]

  useEffect(() => {
    setGraphDim1({
      width: graphRef1?.current?.offsetWidth || 0,
      height: graphRef1?.current?.offsetHeight || 0,
    })
    setGraphDim2({
      width: graphRef2?.current?.offsetWidth || 0,
      height: graphRef2?.current?.offsetHeight || 0,
    })
  }, [graphRef1, graphRef2])

  return (
    <Box display="flex" flexDirection="column" width="100%">
      <Box display="flex" flexDirection="column" width="100%">
        <Card className={styles.overview}>
          <Box display="flex" flexDirection="row">
            <DisplayBox
              title={'PNL (24H)'}
              data={'$564.93'}
              metaText={'$564.93'}
              primaryVariant="h4"
            />
            <Divider />
            <DisplayBox
              title={'PNL (7D)'}
              data={'$564.93'}
              metaText={'$564.93'}
              primaryVariant="h4"
            />{' '}
            <Divider />
            <DisplayBox
              title={'PNL (30D)'}
              data={'$564.93'}
              metaText={'$564.93'}
              primaryVariant="h4"
            />{' '}
          </Box>
        </Card>
        <Box display="flex" width="100%">
          <Box
            display="flex"
            flexDirection="column"
            width="70%"
            minHeight={350}
            ref={graphRef1}
          >
            <CumulativeLineChart
              data={data}
              width={graphDim1.width}
              height={graphDim1.height / 2}
            />
            <DailyBarChart
              data={data}
              width={graphDim1.width}
              height={graphDim1.height / 2}
            />
          </Box>
          <Box
            display="flex"
            flexDirection="column"
            width="30%"
            height="80%"
            minHeight={250}
            ref={graphRef2}
          >
            <AllocPieChart
              data={data2}
              width={graphDim2.width}
              height={graphDim2.height}
            />
          </Box>
        </Box>
      </Box>
    </Box>
  )
}
