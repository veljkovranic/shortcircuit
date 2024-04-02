import { wireTmGrammars } from 'monaco-textmate';
import { Registry } from 'monaco-textmate'; // The registry provides the grammar
import { loadWASM } from 'onigasm'; // Required for syntax highlighting
import React, { useEffect, useRef} from 'react';
import * as monaco from 'monaco-editor';

const Editor = () => {
  useEffect(() => {
    const initializeEditor = async () => {
      await loadWASM('onigasm.wasm');

      const registry = new Registry({
        getGrammarDefinition: async () => ({
          format: 'json',
          content: await (await fetch('grammar.json')).text(),
        }),
      });

      const grammars = new Map();
      grammars.set('myCustomLanguage', 'scopeNameInGrammar');

      const editor = monaco.editor.create(document.getElementById('container'), {
        value: 'your code here',
        language: 'myCustomLanguage',
      });

      await wireTmGrammars(monaco, registry, grammars, editor);
    };

    initializeEditor();
  }, []);

  return <div id="container" style={{ height: '500px' }} />;
};

export default Editor;
