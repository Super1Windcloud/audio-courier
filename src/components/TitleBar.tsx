import React, { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const TitleBar: React.FC = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const window = getCurrentWindow();

  const handleMinimize = async () => {
    await window.hide();
  };

  const handleMaximize = async () => {
    const max = await window.isMaximized();
    if (max) {
      await window.unmaximize();
      setIsMaximized(false);
    } else {
      await window.maximize();
      setIsMaximized(true);
    }
  };

  const handleClose = async () => {
    await window.close();
  };

  return (
    <div
      className="w-full flex justify-end items-center h-10 select-none bg-transparent"
      style={{ WebkitAppRegion: "drag" }}
    >
      <div
        className="flex items-center gap-1 pr-2"
        style={{ WebkitAppRegion: "no-drag" }}
      >
        {/* 最小化 */}
        <button
          onClick={handleMinimize}
          className="w-10 h-8 flex items-center justify-center text-gray-400  dark:hover:bg-gray-700 hover:text-gray-600 rounded transition-colors"
        >
          <svg width="10" height="2" viewBox="0 0 10 2" fill="none">
            <rect width="10" height="2" rx="1" fill="currentColor" />
          </svg>
        </button>

        <button
          onClick={handleMaximize}
          className="w-10 h-8 flex items-center justify-center text-gray-400  dark:hover:bg-gray-700 hover:text-gray-600 rounded transition-colors"
        >
          {isMaximized ? (
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              role="img"
              aria-hidden="false"
              xmlns="http://www.w3.org/2000/svg"
            >
              <title>还原窗口</title>
              <rect
                x="7"
                y="3.5"
                width="13"
                height="13"
                rx="1.4"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                fill="none"
              />
              <rect
                x="3.5"
                y="7"
                width="13"
                height="13"
                rx="1.4"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                fill="none"
              />
            </svg>
          ) : (
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
              <rect
                x="3"
                y="3"
                width="18"
                height="18"
                rx="2"
                stroke="currentColor"
                strokeWidth="1.6"
              />
            </svg>
          )}
        </button>

        {/* 关闭 */}
        <button
          onClick={handleClose}
          className="w-10 h-8 flex items-center justify-center text-gray-400  hover:text-white rounded transition-colors"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
            <path
              d="M6 6L18 18M6 18L18 6"
              stroke="currentColor"
              strokeWidth="1.8"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default TitleBar;
