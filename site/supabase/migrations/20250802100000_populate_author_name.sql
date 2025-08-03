-- Populate author_name field from profiles table
UPDATE public.agents a
SET author_name = COALESCE(p.display_name, p.github_username, 'Unknown')
FROM public.profiles p
WHERE a.user_id = p.user_id
AND a.author_name IS NULL;

-- Set default value for agents without matching profiles
UPDATE public.agents
SET author_name = 'Unknown'
WHERE author_name IS NULL;

-- Make author_name NOT NULL with default
ALTER TABLE public.agents 
ALTER COLUMN author_name SET DEFAULT 'Unknown',
ALTER COLUMN author_name SET NOT NULL;