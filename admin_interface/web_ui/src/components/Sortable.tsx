import {DndContext, DragEndEvent, DragStartEvent, PointerSensor, useSensor, useSensors} from "@dnd-kit/core";
import {arrayMove, SortableContext, useSortable} from "@dnd-kit/sortable";
import {ReactElement, useState} from "react";
import {CSS} from "@dnd-kit/utilities";

interface ItemProps {
  itemId: string;
  children: (props: Record<string, unknown>, isDragging: boolean) => ReactElement;
}

export function SortableItem({itemId, children}: ItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({id: itemId})

  // Fix weird scaling
  if (transform) {
    transform.scaleY = 1;
  }

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: 1,
    outline: 'none'
  }

  if (isDragging) {
    style.zIndex = 999;
  }

  const props = {
    ref: setNodeRef,
    style,
    ...attributes,
    ...listeners,
  }

  return children(props, isDragging);
}


interface SortableProps<T> {
  items: T[];
  onDragEnd: (sortedItems: T[]) => void;
  children: (visibleItems: T[], selectedItemIds: number[], onSelect: (e: React.MouseEvent, id: number) => void) => ReactElement;
  getItemIdCallback: (item: T) => number | string;
}

export function Sortable<T>(props: SortableProps<T>) {
  const {
    items: propsItems,
    onDragEnd,
    children,
    getItemIdCallback = (item) => item.id
  } = props;
  const [selectedItemIds, setSelectedItemIds] = useState<number[]>([]);
  const [draggingId, setDraggingId] = useState(-1);
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  );

  const items = propsItems.map((item, index) => ({...item, id: getItemIdCallback(item)}))
  const visibleItems = items.filter(item => {
    if (draggingId === -1) {
      return true;
    }
    if (item.id === draggingId) {
      return true;
    }
    // Exclude other selected entries beside the dragging one
    return !selectedItemIds.includes(item.id);
  });

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
      setSelectedItemIds(selectedItemIds => [...selectedItemIds, ...(selectedChildren.map(entry => entry.id) || [])]);
    } else if (selectedItemIds.includes(selectedId)) {
      setSelectedItemIds(selectedItemIds.filter(existingId => existingId !== selectedId));
    } else {
      setSelectedItemIds([...selectedItemIds, selectedId]);
    }
  }

  const handleDragEnd = (event: DragEndEvent) => {
    setDraggingId(-1);

    if (event.active.id === event.over?.id) {
      return;
    }

    const movingItems = items.filter(item => selectedItemIds.includes(item.id));
    if (movingItems.length === 0) {
      movingItems.push(items.find(item => item.id == event.active.id))
    }

    const otherItems = items.filter(item => !movingItems.includes(item));

    const targetIndex = otherItems.findIndex(item => item.id == event.over?.id);
    const currentIndex = items.findIndex(item => item.id == event.active.id);
    const isBackward = targetIndex < currentIndex;
    otherItems.splice(targetIndex + (isBackward ? 0 : 1), 0, ...movingItems);

    onDragEnd(otherItems);
    setSelectedItemIds([]);
  }

  const handleDragStart = (event: DragStartEvent) => {
    setDraggingId(event.active.id);
  }

  const handleDragCancel = () => {
    setDraggingId(-1);
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd} onDragStart={handleDragStart} onDragCancel={handleDragCancel}>
      <SortableContext items={items}>
        {children(visibleItems, selectedItemIds, handleSelect)}
      </SortableContext>
    </DndContext>
  )
}