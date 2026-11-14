import { withBase } from '@rspress/core/runtime';
import { useEffect, useState } from 'react';

/** `postMessage` payload sent by the demo page (`src-demo/index.ts`). */
export interface ResizeIframeMessage {
  type: string;
  height: number;
}

/**
 * Embeds the demo application and follows its reported height, so the
 * playground page scrolls as one document instead of inside the iframe.
 */
export function PlaygroundFrame({ src, title }: { src: string; title: string }) {
  const [height, setHeight] = useState('300px');
  // `base` in rstack.config.ts is derived from the package.json name, so
  // `/demo/index.html` becomes `/annealer/demo/index.html`.
  const iframeSource = withBase(src.startsWith('/') ? src : `/${src}`);

  useEffect(() => {
    const handleMessage = (event: MessageEvent<ResizeIframeMessage>) => {
      if (event.data.type === 'resize-iframe') {
        setHeight(`${String(event.data.height)}px`);
      }
    };
    window.addEventListener('message', handleMessage);

    return () => {
      window.removeEventListener('message', handleMessage);
    };
  }, []);

  return (
    <iframe
      title={title}
      src={iframeSource}
      style={{
        width: '100%',
        height: height,
        border: 'none',
        overflow: 'hidden',
        transition: 'height 0.2s ease'
      }}
    />
  );
}
