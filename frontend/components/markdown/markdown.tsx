import React from 'react';
import { unified } from 'unified';
import rehypeHighlight from 'rehype-highlight';
import rehypeKatex from 'rehype-katex';
import rehypeReact from 'rehype-react';
import remarkGemoji from 'remark-gemoji';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import MdEditor from 'react-markdown-editor-lite';
import { UploadFunc } from 'react-markdown-editor-lite/cjs/share/var';
import 'react-markdown-editor-lite/lib/index.css';

import './markdown.module.css';

const MarkdownProcessor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkGemoji)
  .use(remarkMath)
  .use(remarkRehype)
  .use(rehypeKatex)
  .use(rehypeHighlight)
  .use(rehypeReact, { createElement: React.createElement })
  .freeze();

export function MarkdownViewer(text: string) {
  return <div>{MarkdownProcessor.processSync(text).result}</div>;
}

interface MarkdownProps {
  content: string;
  editorStyle?: any;
  mode: 'view' | 'edit';
  onChange?: (text: string) => void;
  onImageUpload?: UploadFunc;
  onCustomImageUpload?: (event: any) => Promise<{
    url: string;
    text?: string | undefined;
  }>;
}

const Markdown: React.FC<MarkdownProps> = ({
  content: value,
  editorStyle,
  mode,
  onChange,
  onImageUpload,
  onCustomImageUpload,
}) => {
  const handleEditorChange = ({ text }: { text: string; html: string }) => {
    if (onChange) {
      onChange(text);
    }
  };

  return (
    <>
      {mode === 'view' ? (
        MarkdownViewer(value)
      ) : (
        <MdEditor
          style={editorStyle}
          value={value}
          onChange={handleEditorChange}
          onImageUpload={onImageUpload}
          onCustomImageUpload={onCustomImageUpload}
          renderHTML={(text) => MarkdownViewer(text)}
        />
      )}
    </>
  );
};

export default Markdown;
