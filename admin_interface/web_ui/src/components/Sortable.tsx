import {DndContext, DragEndEvent, PointerSensor, useSensor, useSensors} from "@dnd-kit/core";
import {arrayMove, SortableContext, useSortable} from "@dnd-kit/sortable";
import {ReactElement} from "react";
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
    zIndex: 1
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


interface SortableProps {
  itemIds: string[];
  onDragEnd: (sortedIds: string[]) => void;
  children: ReactElement;
}

export function Sortable({itemIds, onDragEnd, children}: SortableProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  )

  const handleDragEnd = (event: DragEndEvent) => {
    if (event.active.id === event.over?.id) {
      return;
    }

    const oldIndex = itemIds.indexOf(event.active.id as string);
    const newIndex = itemIds.indexOf(event.over?.id as string);

    onDragEnd(arrayMove(itemIds, oldIndex, newIndex));
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
      <SortableContext items={itemIds}>
        {children}
      </SortableContext>
    </DndContext>
  )
}