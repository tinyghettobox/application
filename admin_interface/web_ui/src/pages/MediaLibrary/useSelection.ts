import {useState} from "react";
import {LibraryEntry} from "@db-models/LibraryEntry";

export default function useSelection(items: LibraryEntry[]) {
  const [selectedItemIds, setSelectedItemIds] = useState<number[]>([]);

  const handleSelect = (event: React.MouseEvent, selectedId: number) => {
    event.preventDefault();
    event.stopPropagation();


    if (event.shiftKey && selectedItemIds.length > 0) {
      const lastSelectedId = selectedItemIds[selectedItemIds.length - 1];
      const startIndex = items.findIndex(child => child.id === lastSelectedId) ?? -1;
      const endIndex = items.findIndex(child => child.id === selectedId) ?? -1;

      if (startIndex === -1 || endIndex === -1) {
        console.warn('Could not find start or end index for shift selection');
      }

      const selectedChildren = items.slice(Math.min(startIndex, endIndex), Math.max(startIndex, endIndex) + 1) || [];
      setSelectedItemIds(selectedItemIds => [...selectedItemIds, ...(selectedChildren.map(entry => entry.id!) || [])]);
    } else if (selectedItemIds.includes(selectedId)) {
      setSelectedItemIds(selectedItemIds.filter(existingId => existingId !== selectedId));
    } else {
      setSelectedItemIds([...selectedItemIds, selectedId]);
    }
  };

  const clearSelection = () => setSelectedItemIds([]);

  return {
    selectedItemIds,
    handleSelect,
    clearSelection,
    setSelectedItemIds,
  };
}
