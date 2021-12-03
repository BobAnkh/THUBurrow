import { useState, useEffect } from 'react';

const useWindowDimensions = () => {
  const hasWindow = typeof window !== 'undefined';

  const getWindowDimensions = () => {
    const width = hasWindow ? window.innerWidth : undefined;
    const height = hasWindow ? window.innerHeight : undefined;

    return { width, height };
  };

  const [windowDimensions, setWindowDimensions] = useState(
    getWindowDimensions()
  );

  useEffect(() => {
    if (hasWindow) {
      const handleResize = () => {
        setWindowDimensions(getWindowDimensions());
      };

      window.addEventListener('resize', handleResize);
      return () => window.removeEventListener('resize', handleResize);
    }
  }, [hasWindow]);

  return windowDimensions;
};

export default useWindowDimensions;
