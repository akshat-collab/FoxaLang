import { useRef } from 'react';
import { Upload, FileCode2 } from 'lucide-react';
import './FileUploader.css';

type Props = {
  onLoad: (filename: string, content: string) => void;
  accept?: string;
};

export function FileUploader({ onLoad, accept = '.foxa,.txt,.md' }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleFiles = async (files: FileList | null) => {
    const file = files?.[0];
    if (!file) return;
    const text = await file.text();
    onLoad(file.name, text);
  };

  return (
    <div
      className="uploader"
      onDragOver={(e) => {
        e.preventDefault();
        e.currentTarget.classList.add('drag');
      }}
      onDragLeave={(e) => e.currentTarget.classList.remove('drag')}
      onDrop={(e) => {
        e.preventDefault();
        e.currentTarget.classList.remove('drag');
        void handleFiles(e.dataTransfer.files);
      }}
    >
      <input
        ref={inputRef}
        type="file"
        accept={accept}
        hidden
        onChange={(e) => void handleFiles(e.target.files)}
      />
      <FileCode2 size={18} />
      <span>Drop a .foxa file or</span>
      <button type="button" className="btn btn-ghost btn-sm" onClick={() => inputRef.current?.click()}>
        <Upload size={14} />
        Upload
      </button>
    </div>
  );
}
