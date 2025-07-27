# Deploying Carp API to Vercel

This guide shows how to deploy the Carp API as a serverless function on Vercel.

## Prerequisites

1. **Vercel Account**: Sign up at [vercel.com](https://vercel.com)
2. **Vercel CLI**: Install with `npm i -g vercel`  
3. **Supabase Project**: Set up your database and get credentials
4. **GitHub Repository**: Push your code to GitHub

## Quick Deployment

### Option 1: Deploy from GitHub (Recommended)

1. **Connect Repository**:
   ```bash
   # Push your code to GitHub
   git add .
   git commit -m "Add Vercel serverless deployment"
   git push origin main
   ```

2. **Import to Vercel**:
   - Go to [vercel.com/dashboard](https://vercel.com/dashboard)
   - Click "Import Project"
   - Select your GitHub repository
   - Choose "Import"

3. **Configure Environment Variables**:
   In the Vercel dashboard, add these environment variables:
   ```
   SUPABASE_URL=https://your-project.supabase.co
   SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
   SUPABASE_JWT_SECRET=your-jwt-secret
   JWT_SECRET=your-secure-jwt-secret-key
   ```

4. **Deploy**:
   - Click "Deploy"
   - Your API will be available at `https://your-project.vercel.app`

### Option 2: Deploy with Vercel CLI

1. **Login to Vercel**:
   ```bash
   vercel login
   ```

2. **Deploy**:
   ```bash
   # From project root
   vercel
   
   # Follow the prompts:
   # - Set up and deploy? [Y/n] Y
   # - Which scope? (your account)
   # - Link to existing project? [y/N] N  
   # - What's your project's name? carp-api
   # - In which directory is your code located? ./
   ```

3. **Set Environment Variables**:
   ```bash
   vercel env add SUPABASE_URL
   vercel env add SUPABASE_SERVICE_ROLE_KEY  
   vercel env add SUPABASE_JWT_SECRET
   vercel env add JWT_SECRET
   ```

4. **Redeploy with Environment Variables**:
   ```bash
   vercel --prod
   ```

## Environment Variables

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `SUPABASE_URL` | Your Supabase project URL | `https://abc123.supabase.co` |
| `SUPABASE_SERVICE_ROLE_KEY` | Service role key from Supabase | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...` |
| `SUPABASE_JWT_SECRET` | JWT secret from Supabase project settings | `your-jwt-secret` |
| `JWT_SECRET` | Secret for API token signing | `your-secure-random-string` |

### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CORS_ORIGINS` | Allowed CORS origins | `*` |
| `MAX_FILE_SIZE` | Max upload size in bytes | `104857600` (100MB) |
| `RATE_LIMIT_RPM` | Requests per minute | `60` |
| `RUST_LOG` | Logging level | `info` |

## API Endpoints

Once deployed, your API will be available at:

- **Health Check**: `GET https://your-project.vercel.app/health`
- **Search Agents**: `GET https://your-project.vercel.app/api/v1/agents/search`
- **Download Agent**: `GET https://your-project.vercel.app/api/v1/agents/{name}/{version}/download`
- **Login**: `POST https://your-project.vercel.app/api/v1/auth/login`
- **Publish Agent**: `POST https://your-project.vercel.app/api/v1/agents/publish` (auth required)

## CLI Configuration

Configure your CLI to use the deployed API:

```bash
# Set the registry URL
carp config set registry-url https://your-project.vercel.app

# Or set environment variable
export CARP_REGISTRY_URL=https://your-project.vercel.app
```

## Testing the Deployment

1. **Health Check**:
   ```bash
   curl https://your-project.vercel.app/health
   ```

2. **Search (no auth required)**:
   ```bash
   curl "https://your-project.vercel.app/api/v1/agents/search?q=example"
   ```

3. **CLI Integration**:
   ```bash
   carp search "example"
   ```

## Monitoring and Logs

### View Logs
```bash
# Real-time logs
vercel logs https://your-project.vercel.app

# Or in dashboard
# Go to vercel.com/dashboard -> your-project -> Functions tab
```

### Performance Monitoring
- Vercel provides built-in analytics
- View in Dashboard -> Analytics tab
- Monitor function execution time and errors

## Custom Domain (Optional)

1. **Add Domain in Vercel Dashboard**:
   - Go to Project Settings -> Domains
   - Add your custom domain (e.g., `api.carp-registry.com`)

2. **Configure DNS**:
   - Add CNAME record pointing to `cname.vercel-dns.com`

3. **Update CLI Configuration**:
   ```bash
   carp config set registry-url https://api.carp-registry.com
   ```

## Security Considerations

1. **Environment Variables**: Never commit secrets to git
2. **JWT Secret**: Use a strong, random JWT secret
3. **CORS**: Configure appropriate CORS origins for production
4. **Rate Limiting**: Monitor and adjust rate limits as needed
5. **Supabase RLS**: Ensure Row Level Security policies are configured

## Troubleshooting

### Common Issues

1. **500 Internal Server Error**:
   - Check Vercel function logs
   - Verify environment variables are set
   - Ensure Supabase connection is working

2. **CORS Errors**:
   - Update `CORS_ORIGINS` environment variable
   - Add your frontend domain to the list

3. **Authentication Failures**:
   - Verify `JWT_SECRET` is set correctly
   - Check Supabase JWT secret matches

4. **File Upload Issues**:
   - Check `MAX_FILE_SIZE` setting
   - Verify Supabase storage bucket exists
   - Ensure proper storage permissions

### Debug Commands

```bash
# Check deployment status
vercel ls

# View function logs
vercel logs https://your-project.vercel.app --follow

# Inspect environment variables
vercel env ls

# Force redeploy
vercel --prod --force
```

## Production Checklist

- [ ] Environment variables configured
- [ ] Supabase database migrations applied
- [ ] Storage buckets created with proper permissions
- [ ] Row Level Security (RLS) policies enabled
- [ ] Custom domain configured (optional)
- [ ] Rate limiting tested
- [ ] CLI integration tested
- [ ] Monitoring and alerts configured
- [ ] Backup strategy for Supabase data

## Cost Considerations

- **Vercel**: Free tier includes 100 GB-hours of compute time
- **Supabase**: Free tier includes 500MB database, 1GB file storage
- **Scaling**: Both services offer paid tiers for production workloads

Your API is now deployed and ready for CLI users to connect to!