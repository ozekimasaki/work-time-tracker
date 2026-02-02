import { Settings } from 'lucide-react';

interface HeaderProps {
  onToggleSettings: () => void;
}

export function Header({ onToggleSettings }: HeaderProps) {
  return (
    <header className="header">
      <h1>作業時間記録</h1>
      <button className="icon-btn" onClick={onToggleSettings}>
        <Settings size={20} />
      </button>
    </header>
  );
}
