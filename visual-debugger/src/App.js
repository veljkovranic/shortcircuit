import React, { useState, useRef, useEffect} from 'react';
import Editor from '@monaco-editor/react';
import { Resizable } from 're-resizable';
import 'reactflow/dist/style.css';
import './App.css';
import NestedFlow from './NestedFlow';
import * as monaco from 'monaco-editor';
import { parser } from "lezer-circom"
import CodeEditor from './components/CodeEditor';


function App() {
  monaco.languages.register({ id: "circom" });

  const [activeTab, setActiveTab] = useState(0);
  const [data, setData] = useState(null);

  const [tabs, setTabs] = useState([
    { filename: 'main.circom', content: "//Insert your code here"},
  ]);
  const editorRef = useRef(null);

  useEffect(() => {
    async function fetchData() {
      try {
        const response = await fetch('http://localhost:3030/get_file');
        if (!response.ok) {
          throw new Error('Failed to fetch data');
        }
        const resp = await response.text();
        setData(resp);
        const currentContent = resp; 
  
        const updatedTabs = tabs.map((tab, i) => ({
          ...tab,
          content: i === activeTab ? parser.parse(currentContent) : parser.parse(tab.content),
        }));
        setTabs(updatedTabs);
        
      } catch (error) {
    
      }
    }

    fetchData();
  }, []);

  const changeTab = (index) => {
    if (editorRef.current) { // Check if the editor ref is defined
      const currentContent = editorRef.current.getValue(); // Use the ref to get the current value

      const updatedTabs = tabs.map((tab, i) => ({
        ...tab,
        content: i === activeTab ? currentContent : tab.content,
      }));
      setTabs(updatedTabs);
      setActiveTab(index); 
    }
  };

  return (
    <div className="App" style={{ display: 'flex', flexDirection: 'row', height: '100vh' }}>
      <Resizable
        defaultSize={{
          width: '50%',
          height: '100%',
        }}
        minWidth="20%"
        maxWidth="80%"
        style={{ display: 'flex', flexDirection: 'column', background: '#333333' }}
        handleStyles={{
          right: {
            background: '#1a1a1a',
            width: '10px',
            cursor: 'ew-resize',
          }
        }}
        >
        <div className="editor-container">
          <div className="tabs">
            {tabs.map((tab, index) => (
              <button
                key={index}
                className={`tab ${index === activeTab ? 'active' : ''}`}
                onClick={() => changeTab(index)}
              >
                {tab.filename}
              </button>
            ))}
          </div>
          {/* <Editor
            height="90vh"
            language="circom"
            theme="vs-dark"
            value={tabs[activeTab].content}
            onChange={(ev, value) => {
              const newTabs = [...tabs];
              // newTabs[activeTab].content = value;
              setTabs(newTabs);
            }}
            onMount={(editor, monaco) => {
              editorRef.current = editor; // Assign the editor instance to the ref
            }}
          /> */}
          <CodeEditor style={{height: '100vh'}} />
        </div>
      </Resizable>
      <div className="flow-container" style={{ flex: 1 }}>
        <NestedFlow/>
      </div>
    </div>
  );
}

export default App;
