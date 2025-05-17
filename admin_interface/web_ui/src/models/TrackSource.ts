export type NewTrackSource = { id?: number, title: string, url?: string, file?: Array<number>, spotifyId?: string, spotifyType?: string, };

export type TrackSource = Required<Pick<NewTrackSource, 'id'>> & NewTrackSource;
