import { useCallback, useEffect } from 'react';
import ReactFlow, {
  addEdge,
  Background,
  useNodesState,
  useEdgesState,
  MiniMap,
  Controls,
} from 'reactflow';
import 'reactflow/dist/style.css';
import CustomNode from './components/CustomNode';

const nodeTypes = {
  customNode: CustomNode,
};


const initialNodes = [
  {
    id: '1',
    type: 'input',
    data: { label: 'Node 0' },
    position: { x: 250, y: 5 },
    className: 'light',
  },
  {
    id: '2',
    data: { label: 'Group A' },
    position: { x: 100, y: 100 },
    className: 'light',
    style: { backgroundColor: 'rgba(255, 0, 0, 0.2)', width: 200, height: 200 },
  },
  {
    id: '2a',
    data: { label: 'Node A.1' },
    position: { x: 10, y: 50 },
    parentNode: '2',
  },
  {
    id: '3',
    data: { label: 'Node 1' },
    position: { x: 320, y: 100 },
    className: 'light',
  },
  {
    id: '4',
    data: { label: 'Group B' },
    position: { x: 320, y: 200 },
    className: 'light',
    style: { backgroundColor: 'rgba(255, 0, 0, 0.2)', width: 300, height: 300 },
    type: 'group',
  },
  {
    id: '4a',
    data: { label: 'Node B.1' },
    position: { x: 15, y: 65 },
    className: 'light',
    parentNode: '4',
    extent: 'parent',
  },
  {
    id: '4b',
    data: { label: 'Group B.A' },
    position: { x: 15, y: 120 },
    className: 'light',
    style: {
      backgroundColor: 'rgba(255, 0, 255, 0.2)',
      height: 150,
      width: 270,
    },
    parentNode: '4',
  },
  {
    id: '4b1',
    data: { label: 'Node B.A.1' },
    position: { x: 20, y: 40 },
    className: 'light',
    parentNode: '4b',
  },
  {
    id: '4b2',
    data: { label: 'Node B.A.2' },
    position: { x: 100, y: 100 },
    className: 'light',
    parentNode: '4b',
  },
];

const initialEdges = [
  { id: 'e1-2', source: '1', target: '2', animated: true },
  { id: 'e1-3', source: '1', target: '3' },
  { id: 'e2a-4a', source: '2a', target: '4a' },
  { id: 'e3-4b', source: '3', target: '4b' },
  { id: 'e4a-4b1', source: '4a', target: '4b1' },
  { id: 'e4a-4b2', source: '4a', target: '4b2' },
  { id: 'e4b1-4b2', source: '4b1', target: '4b2' },
];

const fetchGraphData = async (component) => {
  try {
    const encodedComponent = encodeURIComponent(component);
    const url = `http://0.0.0.0:3030/graph-data/${encodedComponent}`;
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return await response.json();
  } catch (error) {
    console.error('Fetching graph data failed:', error);
  }
};


const NestedFlow = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const update_nodes = async (component) => {
    const graphData = await fetchGraphData(component);
        console.log(graphData);
        for (let node in graphData.initialNodes) {
          if (node.type == 'customNode') {
            console.log("CUSTOM NODE FIDDLE");
            node.data = { label:node.data.label, onChildAction: fetchGraphData }
          }
        }
  
        if (graphData) {
          setNodes(graphData.initialNodes);
          setEdges(graphData.initialEdges);
        }
  };

  useEffect(() => {
    const loadGraphData = async () => {
      const graphData = await fetchGraphData('main');
      for (let node in graphData.initialNodes) {
        if (graphData.initialNodes[node].type == "customNode") {
          console.log("CUSTOM NODE FIDDLE");
          graphData.initialNodes[node].data = { label:graphData.initialNodes[node].data.label, onChildAction: update_nodes }
          console.log(graphData.initialNodes[node]);

        }
      }

      if (graphData) {
        setNodes(graphData.initialNodes);
        setEdges(graphData.initialEdges);
      }
    };

    loadGraphData();
  }, []);

  const onConnect = useCallback((connection) => {
    setEdges((eds) => addEdge(connection, eds));
  }, []);

  return (
    <ReactFlow
      nodeTypes={nodeTypes} 
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      className="react-flow-subflows-example"
      fitView
    >
      {/* <MiniMap /> */}
      <Controls />
      <Background />
    </ReactFlow>
  );
};

export default NestedFlow;