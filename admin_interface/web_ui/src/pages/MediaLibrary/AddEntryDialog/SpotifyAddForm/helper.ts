import {LibraryEntry} from "@db-models/LibraryEntry";
import {cropImage} from "@/pages/MediaLibrary/AddEntryDialog/helper";

interface SearchResultItem {
  id: string,
  name: string,
  images?: { url: string }[],
  type: string,
}

export async function searchResultToLibraryEntry(item: SearchResultItem, type: string, sortKey: number): Promise<LibraryEntry> {
  let image: number[] | undefined = undefined;
  if (item.images?.[0]?.url) {
    const response = await fetch(item.images?.[0]?.url);
    if (!response.ok) {
      throw new Error(`Failed to fetch image: ${response.statusText}`);
    }
    image = await cropImage(await response.blob(), 180, 180, 0.8);
  }

  return {
    name: item.name,
    variant: type === 'track' || type === 'episode' ? 'spotify' : 'folder',
    image,
    sortKey,
    trackSource: {
      title: item.name,
      spotifyId: item.id,
      spotifyType: type
    }
  }
}