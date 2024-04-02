import React from 'react';
import { useCodeMirror } from '@uiw/react-codemirror';
import { javascript } from '@codemirror/lang-javascript';
import { basicSetup } from '@uiw/react-codemirror';
import { circomLanguage } from "codemirror-lang-circom"

function CodeEditor() {
  const [code, setCode] = React.useState("// Start coding...\n");
  const { setContainer } = useCodeMirror({
    value: code,
    extensions: [javascript(), circomLanguage],
    onChange: (value, viewUpdate) => {
      setCode(value);
    },
  });
  React.useEffect(() => {
    async function fetchData() {
      try {
        const response = await fetch('http://localhost:3030/get_file');
        if (!response.ok) {
          throw new Error('Failed to fetch data');
        }
        const resp = await response.text();
        setCode(resp);
      } catch (error) {
    
      }
    }

    fetchData();
  }, []);
  React.useEffect(() => {
    setContainer(document.getElementById('codemirror'));
    return () => setContainer();
  }, [setContainer]);

  return <div id="codemirror" style={{ height: '100vh', textAlign:'left'}}></div>;
}

export default CodeEditor;
