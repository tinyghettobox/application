import {DndContext, DragEndEvent, DragStartEvent, PointerSensor, useSensor, useSensors} from "@dnd-kit/core";
import {SortableContext, useSortable} from "@dnd-kit/sortable";
import {ReactElement, useState} from "react";
import {CSS} from "@dnd-kit/utilities";

interface ItemProps {
  itemId: number | string;
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
  children: (visibleItems: T[]) => ReactElement;
  getItemId: (item: T) => number | string;
  selectedItemIds?: (number | string)[];
}

export function Sortable<T>(props: SortableProps<T>) {
  const {
    items: propsItems,
    onDragEnd,
    children,
    getItemId,
    selectedItemIds = [],
  } = props;
  const [draggingId, setDraggingId] = useState(-1);
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  );

  const items = propsItems.map((item, index) => ({...item, id: getItemId(item)}))
  const visibleItems = items.filter(item => {
    if (draggingId === -1) {
      return true;
    }
    if (getItemId(item) === draggingId) {
      return true;
    }
    // Exclude other selected entries beside the dragging one
    return !selectedItemIds.includes(getItemId(item));
  });

  const handleDragEnd = (event: DragEndEvent) => {
    setDraggingId(-1);

    if (event.active.id === event.over?.id) {
      return;
    }

    const movingItems = items.filter(item => selectedItemIds.includes(getItemId(item)));
    if (movingItems.length === 0) {
      const activeItem = items.find(item => getItemId(item) == event.active.id);
      if (activeItem) {
        movingItems.push(activeItem);
      }
    }

    const otherItems = items.filter(item => !movingItems.includes(item));

    const targetIndex = otherItems.findIndex(item => getItemId(item) == event.over?.id);
    const currentIndex = items.findIndex(item => getItemId(item) == event.active.id);
    const isBackward = targetIndex < currentIndex;
    otherItems.splice(targetIndex + (isBackward ? 0 : 1), 0, ...movingItems);

    onDragEnd(otherItems);
  }

  const handleDragStart = (event: DragStartEvent) => {
    setDraggingId(Number(event.active.id));
  }

  const handleDragCancel = () => {
    setDraggingId(-1);
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd} onDragStart={handleDragStart} onDragCancel={handleDragCancel}>
      <SortableContext items={items}>
        {children(visibleItems)}
      </SortableContext>
    </DndContext>
  )
}
