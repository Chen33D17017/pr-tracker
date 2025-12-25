import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Search, 
  Filter, 
  GitPullRequest, 
  CheckCircle2, 
  Clock, 
  AlertCircle, 
  MoreHorizontal, 
  ChevronRight, 
  X, 
  ExternalLink, 
  MessageSquare, 
  GitBranch, 
  Github, 
  LayoutDashboard, 
  Inbox, 
  Settings, 
  Plus, 
  User, 
  Key, 
  Save, 
  Trash2, 
  FolderPlus, 
  Star, 
  Check, 
  Trophy, 
  Link,
  Eye,
  EyeOff,
  RefreshCw,
  BarChart3,
  TrendingUp,
  Users,
  Zap,
  Wifi,
  WifiOff,
  Copy
} from 'lucide-react';

// TypeScript interfaces for database structures
interface Project {
  id: number;
  name: string;
  description: string | null;
  created_at: number;
}

interface PullRequest {
  id: number;
  github_id: number;
  pr_number: number;
  title: string | null;
  author_id: number;
  project_id: number | null;
  last_updated_at: number;
  author_name: string | null;
  author_avatar: string | null;
  author_display_name: string | null;
  project_name: string | null;
  status: string;
  branch: string | null;
  score: number | null;
  repository_owner: string | null;
  repository_name: string | null;
}

// 初始化模擬數據
// Removed INITIAL_PRS mock data - now using real database data

const App = () => {
  const [prs, setPrs] = useState<PullRequest[]>([]);  // Now uses real database data only
  const [selectedPrId, setSelectedPrId] = useState(null);
  const [activeTab, setActiveTab] = useState('Overview');
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState(null); 
  const [repoFilter, setRepoFilter] = useState(null); 
  
  // 視窗與測試狀態
  const [showSettings, setShowSettings] = useState(false);

  // Load GitHub token when settings modal opens
  const handleOpenSettings = async () => {
    setShowSettings(true);
    try {
      const storedToken = await invoke('get_github_token');
      if (storedToken) {
        setApiKey(storedToken);
      }
    } catch (error) {
      console.error('Failed to load GitHub token:', error);
    }
  };
  const [showAddProject, setShowAddProject] = useState(false);
  const [showNewPr, setShowNewPr] = useState(false);
  const [showScoring, setShowScoring] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isTesting, setIsTesting] = useState(false); // 是否正在測試連線
  const [testStatus, setTestStatus] = useState(null); // 'success' | 'error' | null
  
  // 數據狀態
  const [apiKey, setApiKey] = useState('');
  const [showKey, setShowKey] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [projects, setProjects] = useState<Project[]>([]);
  const [dbInitialized, setDbInitialized] = useState(false);
  const [isLoadingProjects, setIsLoadingProjects] = useState(false);
  const [projectToDelete, setProjectToDelete] = useState<Project | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [errorModal, setErrorModal] = useState<{show: boolean, title: string, message: string}>({
    show: false,
    title: '',
    message: ''
  });

  // 新 PR 表單狀態
  const [newPrData, setNewPrData] = useState({ url: '', project: '' });

  // 評分狀態
  const [hoverScore, setHoverScore] = useState(0);
  const [selectedScore, setSelectedScore] = useState(0);

  const selectedPr = prs.find(p => p.id === selectedPrId);

  // Error handling utility
  const showError = (title: string, message: string) => {
    setErrorModal({
      show: true,
      title,
      message
    });
  };

  // Success notification utility
  const showSuccess = (title: string, message: string) => {
    setErrorModal({
      show: true,
      title,
      message
    });
  };

  // Copy GitHub PR URL to clipboard
  const handleCopyGitHubUrl = async (pr: PullRequest) => {
    if (pr.repository_owner && pr.repository_name && pr.pr_number) {
      const githubUrl = `https://github.com/${pr.repository_owner}/${pr.repository_name}/pull/${pr.pr_number}`;
      try {
        await navigator.clipboard.writeText(githubUrl);
        // Could add a toast notification here if desired
      } catch (error) {
        console.error('❌ Failed to copy GitHub URL to clipboard:', error);
        showError('Copy Failed', 'Failed to copy GitHub URL to clipboard.');
      }
    } else {
      showError('Missing Repository Information', 'This PR does not have repository information stored. Please re-add the PR from the GitHub URL.');
    }
  };

  // Database initialization and project management functions
  const initializeDatabase = async () => {
    try {
      await invoke('init_database');
      setDbInitialized(true);
    } catch (error) {
      console.error('Failed to initialize database:', error);
    }
  };

  const loadProjects = async () => {
    try {
      setIsLoadingProjects(true);
      const projectList = await invoke('get_projects') as Project[];

      // Sort projects by creation date (latest first)
      const sortedProjects = projectList.sort((a, b) => b.created_at - a.created_at);

      setProjects(sortedProjects);

      // Update newPrData.project if it's empty and we have projects
      if (!newPrData.project && sortedProjects.length > 0) {
        setNewPrData(prev => ({ ...prev, project: sortedProjects[0].name }));
      }
    } catch (error) {
      console.error('Failed to load projects:', error);
    } finally {
      setIsLoadingProjects(false);
    }
  };

  const addNewProject = async (name: string, description?: string) => {
    try {
      console.log('➕ Adding new project:', { name, description });
      const newProject = await invoke('add_project', {
        name,
        description: description || null
      }) as Project;

      // Add new project at the beginning (latest first)
      setProjects(prev => [newProject, ...prev]);

      return newProject;
    } catch (error) {
      console.error('Failed to add project:', error);
      throw error;
    }
  };

  const updateExistingProject = async (id: number, name: string, description?: string) => {
    try {
      const updatedProject = await invoke('update_project', {
        id,
        name,
        description: description || null
      }) as Project;
      setProjects(prev => prev.map(p => p.id === id ? updatedProject : p));
      return updatedProject;
    } catch (error) {
      console.error('Failed to update project:', error);
      throw error;
    }
  };

  const deleteExistingProject = async (id: number) => {
    try {
      await invoke('delete_project', { id });
      setProjects(prev => prev.filter(p => p.id !== id));
    } catch (error) {
      console.error('Failed to delete project:', error);
      throw error;
    }
  };

  const checkProjectHasPRs = (projectName: string): boolean => {
    // Check against database PRs instead of dummy data
    return prs.some(pr => pr.project_name === projectName);
  };

  const handleDeleteProject = (project: Project) => {
    setProjectToDelete(project);
    setShowDeleteConfirm(true);
  };

  const confirmDeleteProject = async () => {
    if (!projectToDelete) return;

    try {
      await deleteExistingProject(projectToDelete.id);
      setShowDeleteConfirm(false);
      setProjectToDelete(null);
    } catch (error) {
      console.error('Failed to delete project:', error);
      showError('Delete Failed', 'Failed to delete project. Please try again.');
    }
  };

  // Initialize database on component mount
  useEffect(() => {
    initializeDatabase();
  }, []);

  // Load projects when database is initialized
  useEffect(() => {
    if (dbInitialized) {
      loadProjects();
      loadPRs();
    }
  }, [dbInitialized]);

  const loadPRs = async () => {
    try {
      const prList = await invoke('get_pull_requests') as PullRequest[];
      setPrs(prList);
    } catch (error) {
      console.error('Failed to load PRs from database:', error);
      // Fallback to empty array if database PRs fail to load
      setPrs([]);
    }
  };

  // Clear all data from database (for testing/cleanup)
  const clearDatabase = async () => {
    try {
      await invoke('clear_all_data');
      // Reload data after clearing
      setPrs([]);
      setProjects([]);
      loadProjects();
      loadPRs();
    } catch (error) {
      console.error('❌ Failed to clear database:', error);
      showError('Database Clear Failed', 'Failed to clear database: ' + error);
    }
  };

  // 動態計算各狀態數量
  const stats = {
    Waiting: prs.filter(p => p.status === 'Waiting').length,
    Reviewing: prs.filter(p => p.status === 'Reviewing').length,
    Action: prs.filter(p => p.status === 'Action').length,
    Approved: prs.filter(p => p.status === 'Approved').length,
  };

  // 複合篩選清單邏輯
  const filteredPrs = prs.filter(pr => {
    const matchesSearch = (pr.title || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
                          pr.id.toString().toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = statusFilter ? pr.status === statusFilter : true;
    const matchesRepo = repoFilter ? pr.project_name === repoFilter : true;
    const notArchived = pr.status !== 'archived'; // Hide archived PRs from main view
    return matchesSearch && matchesStatus && matchesRepo && notArchived;
  });

  // 計算 Performance 數據
  const calculatePerformance = () => {
    const perfMap = {};
    prs.forEach(pr => {
      const name = pr.author_name || 'Unknown';
      if (!perfMap[name]) {
        perfMap[name] = {
          name,
          avatar: pr.author_avatar,
          approvedCount: 0,
          totalScore: 0,
          scoredCount: 0
        };
      }
      if (pr.status === 'Approved') {
        perfMap[name].approvedCount += 1;
      }
      if (pr.score !== null) {
        perfMap[name].totalScore += pr.score;
        perfMap[name].scoredCount += 1;
      }
    });

    return Object.values(perfMap).map(person => ({
      ...person,
      avgScore: person.scoredCount > 0 ? (person.totalScore / person.scoredCount).toFixed(1) : "N/A"
    })).sort((a, b) => {
      // Handle N/A values - put them at the end (lowest ranking)
      if (a.avgScore === "N/A" && b.avgScore === "N/A") return 0;
      if (a.avgScore === "N/A") return 1; // a goes to end
      if (b.avgScore === "N/A") return -1; // b goes to end
      // Normal numeric sorting for valid scores (highest first)
      return b.avgScore - a.avgScore;
    });
  };

  const performanceList = calculatePerformance();

  const getStatusStyle = (status) => {
    switch (status) {
      case 'Waiting': return 'bg-amber-50 text-amber-700 border-amber-200';
      case 'Reviewing': return 'bg-blue-50 text-blue-700 border-blue-200';
      case 'Action': return 'bg-rose-50 text-rose-700 border-rose-200';
      case 'Approved': return 'bg-emerald-50 text-emerald-700 border-emerald-200';
      default: return 'bg-gray-50 text-gray-600 border-gray-200';
    }
  };

  const handleStatusCardClick = (status) => {
    if (statusFilter === status) {
      setStatusFilter(null);
    } else {
      setStatusFilter(status);
      setActiveTab('Overview');
    }
  };

  const handleProjectClick = (project) => {
    if (repoFilter === project) {
      setRepoFilter(null);
    } else {
      setRepoFilter(project);
      setActiveTab('Overview');
    }
  };

  const handleTestConnection = async () => {
    if (!apiKey) return;
    setIsTesting(true);
    setTestStatus(null);

    try {
      // Test the actual GitHub token with the API
      const result = await invoke('verify_github_token', { token: apiKey });
      setIsTesting(false);
      setTestStatus('success');
      console.log('GitHub token verified:', result);
    } catch (error) {
      setIsTesting(false);
      setTestStatus('error');
      console.error('GitHub token verification failed:', error);
    }
  };

  const handleSaveSettings = async () => {
    try {
      if (apiKey) {
        // Save the GitHub token to keychain
        await invoke('save_github_token', { token: apiKey });
        console.log('GitHub token saved successfully');
      }
      setShowSettings(false);
      setTestStatus(null);
    } catch (error) {
      console.error('Failed to save GitHub token:', error);
      showError('Token Save Failed', `Failed to save GitHub token: ${error}`);
    }
  };

  const updatePrStatus = async (id, newStatus) => {
    try {
      // Update backend first
      await invoke('update_pr_status', { prId: id, status: newStatus });

      // Update local state only if backend update succeeds
      setPrs(prs.map(p => p.id === id ? { ...p, status: newStatus } : p));
    } catch (error) {
      console.error('Failed to update PR status:', error);
      showError('Update Failed', 'Failed to update PR status: ' + String(error));
    }
  };

  const updatePrProject = async (id, newProjectName) => {
    try {
      // Find the project ID by name
      const project = projects.find(p => p.name === newProjectName);
      if (!project) {
        showError('Update Failed', 'Project not found');
        return;
      }

      // Update backend first
      await invoke('update_pr_project', { prId: id, projectId: project.id });

      // Update local state only if backend update succeeds
      setPrs(prs.map(p => p.id === id ? {
        ...p,
        project_name: newProjectName,
        project_id: project.id
      } : p));
    } catch (error) {
      console.error('Failed to update PR project:', error);
      showError('Update Failed', 'Failed to update PR project assignment: ' + String(error));
    }
  };

  const handleSyncBranch = () => {
    setIsSyncing(true);
    setTimeout(() => setIsSyncing(false), 1500);
  };

  const copyBranchDetails = async (pr) => {
    try {
      const copyText = `cr into main from ${pr.branch}`;
      await navigator.clipboard.writeText(copyText);
      console.log('Copied to clipboard:', copyText);
      // Could add a toast notification here if desired
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  };

  const archivePr = async (id) => {
    try {
      // Update backend to set status to "archived"
      await invoke('update_pr_status', { prId: id, status: 'archived' });

      // Update local state
      setPrs(prs.map(p => p.id === id ? { ...p, status: 'archived' } : p));

      // Close the modal since the PR is now archived
      setSelectedPrId(null);
    } catch (error) {
      console.error('Failed to archive PR:', error);
      showError('Archive Failed', 'Failed to archive PR: ' + String(error));
    }
  };

  const isValidGitHubPRUrl = (url: string): boolean => {
    const githubPrPattern = /^https:\/\/github\.com\/[\w-]+\/[\w.-]+\/pull\/\d+/;
    return githubPrPattern.test(url);
  };

  const extractTitleFromPRUrl = (url: string): string => {
    // Extract basic info from URL for now - we'll enhance this with GitHub API later
    const match = url.match(/github\.com\/([\w-]+)\/([\w.-]+)\/pull\/(\d+)/);
    if (match) {
      const [, owner, repo, prNumber] = match;
      return `PR #${prNumber} from ${owner}/${repo}`;
    }
    return 'GitHub PR';
  };

  const submitNewPr = async (e) => {
    e.preventDefault();
    if (!isValidGitHubPRUrl(newPrData.url)) {
      showError('Invalid GitHub URL', 'Please enter a valid GitHub PR URL (e.g., https://github.com/owner/repo/pull/123)');
      return;
    }
    let selectedProject = projects.find(p => p.name === newPrData.project);

    // If the stored project name doesn't exist, use the first available project
    if (!selectedProject && projects.length > 0) {
      selectedProject = projects[0];
      // Update the form to reflect the correct project
      setNewPrData(prev => ({ ...prev, project: selectedProject.name }));
    }

    if (!selectedProject) {
      showError('No Project Available', 'Please create a project first before adding PRs');
      return;
    }

    try {
      // Check if we have the token in memory (from settings)
      if (!apiKey) {
        showError('GitHub Token Required', 'Please set and save your GitHub token in Settings first.');
        return;
      }

      // First test if Tauri invoke is working at all
      try {
        const testResult = await invoke('test_invoke', { message: 'Hello from frontend' }) as string;
      } catch (testError) {
        console.error('Tauri invoke test failed:', testError);
        showError('Tauri Invoke Error', 'Basic Tauri communication is failing: ' + String(testError));
        return;
      }

      const newPR = await invoke('add_pr_from_github_url', {
        prUrl: newPrData.url,
        projectId: selectedProject.id,
        token: apiKey
      }) as PullRequest;

      // Close modal and reset form
      setShowNewPr(false);
      setNewPrData({ url: '', project: projects.length > 0 ? projects[0].name : '' });

      // Reload PRs in the background
      loadPRs().catch((error) => {
        console.error('Error reloading PRs:', error);
      });
    } catch (error) {
      console.error('Failed to add PR:', error);
      showError('Failed to Add PR', String(error));
    }
  };

  const submitScore = async () => {
    try {
      // Update both status and score in the backend
      await invoke('update_pr_status', { prId: selectedPrId, status: 'Approved' });
      await invoke('update_pr_score', { prId: selectedPrId, score: selectedScore });

      // Update local state only if backend updates succeed
      setPrs(prs.map(p => p.id === selectedPrId ? { ...p, status: 'Approved', score: selectedScore } : p));

      setShowScoring(false);
      setSelectedScore(0);
    } catch (error) {
      console.error('Failed to submit score:', error);
      showError('Update Failed', 'Failed to approve PR and save score: ' + String(error));
    }
  };

  const handleAddProject = async (e) => {
    e.preventDefault();

    if (newProjectName.trim()) {
      try {
        const createdProject = await addNewProject(newProjectName.trim());

        setNewProjectName('');
        setShowAddProject(false);

        // Refresh project list to show the new project
        await loadProjects();
      } catch (error) {
        console.error('❌ Failed to add project in form handler:', error);
        showError('Project Creation Failed', `Failed to create project: ${error}`);
      }
    } else {
    }
  };

  return (
    <div className="flex h-screen bg-[#F9FAFB] font-sans text-slate-900 antialiased overflow-hidden">
      {/* Sidebar */}
      <aside className="w-64 bg-white border-r border-slate-200 flex flex-col hidden md:flex">
        <div className="p-6 flex items-center gap-3">
          <div className="bg-indigo-600 p-1.5 rounded-lg text-white shadow-sm">
            <GitPullRequest size={20} />
          </div>
          <span className="font-bold text-lg tracking-tight">PR Manager</span>
        </div>
        
        <nav className="flex-1 px-4 space-y-1 overflow-y-auto">
          <NavItem 
            icon={<LayoutDashboard size={18}/>} 
            label="Overview" 
            active={activeTab === 'Overview' && !statusFilter && !repoFilter} 
            onClick={() => { setActiveTab('Overview'); setStatusFilter(null); setRepoFilter(null); }} 
          />
          <NavItem 
            icon={<Inbox size={18}/>} 
            label="Waiting for me" 
            badge={stats.Waiting.toString()} 
            active={statusFilter === 'Waiting'}
            onClick={() => { setActiveTab('Overview'); setStatusFilter('Waiting'); setRepoFilter(null); }} 
          />
          <NavItem 
            icon={<BarChart3 size={18}/>} 
            label="Performance" 
            active={activeTab === 'Performance'}
            onClick={() => { setActiveTab('Performance'); setStatusFilter(null); setRepoFilter(null); }} 
          />
          
          <div className="py-4 px-2">
            <div className="flex items-center justify-between mb-2 px-2">
              <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider text-[10px]">Projects</h3>
              <button onClick={() => setShowAddProject(true)} className="p-1 hover:bg-slate-100 rounded text-slate-400 hover:text-indigo-600 transition-colors">
                <Plus size={14} />
              </button>
            </div>
            <div className="space-y-0.5">
              {isLoadingProjects ? (
                <div className="text-center py-2">
                  <span className="text-xs text-slate-400">Loading projects...</span>
                </div>
              ) : projects.length > 0 ? (
                <>
                  {projects.map((project) => {
                    const hasPRs = checkProjectHasPRs(project.name);
                    return (
                      <div key={project.id} className="group flex items-center">
                        <NavItem
                          label={project.name}
                          active={repoFilter === project.name}
                          onClick={() => handleProjectClick(project.name)}
                          small
                          icon={<ChevronRight size={14}/>}
                          badge=""
                        />
                        {!hasPRs && (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDeleteProject(project);
                            }}
                            className="opacity-0 group-hover:opacity-100 p-1 ml-1 hover:bg-red-50 hover:text-red-600 rounded text-slate-400 transition-all"
                            title={`Delete project: ${project.name}`}
                          >
                            <Trash2 size={12} />
                          </button>
                        )}
                        {hasPRs && (
                          <div
                            className="opacity-0 group-hover:opacity-60 p-1 ml-1 text-slate-300 cursor-not-allowed"
                            title={`Cannot delete "${project.name}" - has assigned PRs`}
                          >
                            <Trash2 size={12} />
                          </div>
                        )}
                      </div>
                    );
                  })}
                </>
              ) : (
                <>
                  <div className="text-center py-2">
                    <span className="text-xs text-slate-400 italic">No projects yet</span>
                  </div>
                </>
              )}
            </div>
          </div>
        </nav>

        <div className="p-4 border-t border-slate-100">
          <NavItem icon={<Settings size={18}/>} label="Settings" onClick={handleOpenSettings} />
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col relative overflow-hidden text-slate-900">
        <header className="bg-white border-b border-slate-200 px-8 py-4 flex items-center justify-between shrink-0">
          <div>
            <h1 className="text-xl font-bold">
              {activeTab === 'Performance' ? 'Team Performance' : 'Pull Requests'}
            </h1>
            <p className="text-sm text-slate-500">
              {activeTab === 'Performance' ? 'Insights on code review metrics' : `Managing ${prs.filter(p => p.status !== 'archived').length} active requests`}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowNewPr(true)}
              className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-lg text-sm font-medium transition-all shadow-sm active:scale-95"
            >
              <Plus size={18} /> New PR
            </button>
          </div>
        </header>

        <div className="p-8 space-y-6 overflow-y-auto">
          {activeTab === 'Performance' ? (
            <div className="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300">
              {/* Performance Cards */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm flex items-center gap-4">
                  <div className="p-3 bg-emerald-50 rounded-xl text-emerald-600">
                    <CheckCircle2 size={24} />
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Total Approved</p>
                    <p className="text-2xl font-black">{stats.Approved}</p>
                  </div>
                </div>
                <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm flex items-center gap-4">
                  <div className="p-3 bg-amber-50 rounded-xl text-amber-600">
                    <Trophy size={24} />
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Avg. Quality</p>
                    <p className="text-2xl font-black">
                      {(prs.filter(p => p.score).reduce((acc, p) => acc + p.score, 0) / prs.filter(p => p.score).length || 0).toFixed(1)}
                    </p>
                  </div>
                </div>
                <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm flex items-center gap-4">
                  <div className="p-3 bg-indigo-50 rounded-xl text-indigo-600">
                    <Users size={24} />
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Active Members</p>
                    <p className="text-2xl font-black">{performanceList.length}</p>
                  </div>
                </div>
              </div>

              {/* Ranking Table */}
              <div className="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
                <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between">
                  <h3 className="font-bold">Member Performance Ranking</h3>
                  <TrendingUp size={18} className="text-slate-400" />
                </div>
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="bg-slate-50/50 text-slate-400 text-[10px] uppercase tracking-wider font-bold">
                      <th className="px-8 py-4">Developer</th>
                      <th className="px-6 py-4">Approved PRs</th>
                      <th className="px-6 py-4">Avg. Score</th>
                      <th className="px-8 py-4 text-right">Performance Rank</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {performanceList.map((person, idx) => (
                      <tr key={person.name} className="hover:bg-slate-50/50 transition-colors">
                        <td className="px-8 py-5">
                          <div className="flex items-center gap-3">
                            <img src={person.avatar} className="w-9 h-9 rounded-full border border-slate-200" />
                            <span className="font-semibold">{person.name}</span>
                          </div>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center gap-2">
                            <span className="text-lg font-black">{person.approvedCount}</span>
                            <span className="text-xs text-slate-400 font-medium tracking-tight">PRs</span>
                          </div>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center gap-3">
                            <span className={`text-sm font-black px-2 py-0.5 rounded ${person.avgScore >= 9 ? 'text-emerald-600 bg-emerald-50' : 'text-indigo-600 bg-indigo-50'}`}>
                              {person.avgScore}
                            </span>
                            <div className="flex-1 max-w-[100px] h-1.5 bg-slate-100 rounded-full overflow-hidden">
                              <div 
                                className={`h-full transition-all duration-1000 ${person.avgScore >= 9 ? 'bg-emerald-500' : 'bg-indigo-500'}`} 
                                style={{ width: `${(person.avgScore / 10) * 100}%` }}
                              />
                            </div>
                          </div>
                        </td>
                        <td className="px-8 py-5 text-right">
                          {idx === 0 && <span className="inline-flex items-center gap-1 text-[10px] font-bold bg-amber-50 text-amber-700 px-2 py-1 rounded-full uppercase tracking-tighter shadow-sm border border-amber-100">Top Performer 🔥</span>}
                          {idx !== 0 && <span className="text-xs font-bold text-slate-300">#{idx + 1}</span>}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          ) : (
            <>
              {/* Stat Cards Filters */}
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
                <StatCard 
                  label="Waiting" 
                  value={stats.Waiting} 
                  icon={<Clock size={20} className="text-amber-500" />} 
                  color="bg-amber-50" 
                  isActive={statusFilter === 'Waiting'}
                  onClick={() => handleStatusCardClick('Waiting')}
                />
                <StatCard 
                  label="Reviewing" 
                  value={stats.Reviewing} 
                  icon={<Search size={20} className="text-blue-500" />} 
                  color="bg-blue-50" 
                  isActive={statusFilter === 'Reviewing'}
                  onClick={() => handleStatusCardClick('Reviewing')}
                />
                <StatCard 
                  label="Action" 
                  value={stats.Action} 
                  icon={<AlertCircle size={20} className="text-rose-500" />} 
                  color="bg-rose-50" 
                  isActive={statusFilter === 'Action'}
                  onClick={() => handleStatusCardClick('Action')}
                />
                <StatCard 
                  label="Approved" 
                  value={stats.Approved} 
                  icon={<CheckCircle2 size={20} className="text-emerald-500" />} 
                  color="bg-emerald-50" 
                  isActive={statusFilter === 'Approved'}
                  onClick={() => handleStatusCardClick('Approved')}
                />
              </div>

              {/* Filter Display */}
              <div className="flex items-center justify-between gap-4 bg-white p-2 rounded-xl border border-slate-200 shadow-sm">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={18} />
                  <input 
                    type="text" 
                    placeholder="Search by title, ID or repo..."
                    className="w-full pl-10 pr-4 py-2 bg-transparent border-none focus:ring-0 text-sm"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
                </div>
                <div className="flex items-center gap-2 pr-2">
                  {(statusFilter || repoFilter) && <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mr-1">Filtered by:</span>}
                  {repoFilter && (
                    <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-indigo-50 text-indigo-700 text-[10px] font-bold border border-indigo-100">
                      PROJECT: {repoFilter.toUpperCase()}
                      <button onClick={() => setRepoFilter(null)} className="hover:text-indigo-900"><X size={12} /></button>
                    </div>
                  )}
                  {statusFilter && (
                    <div className={`flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-bold border ${getStatusStyle(statusFilter)}`}>
                      STATUS: {statusFilter.toUpperCase()}
                      <button onClick={() => setStatusFilter(null)} className="hover:opacity-60"><X size={12} /></button>
                    </div>
                  )}
                </div>
              </div>

              {/* Table */}
              <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="bg-slate-50/50 text-slate-400 text-[11px] uppercase tracking-wider font-bold">
                      <th className="px-6 py-4">Pull Request Info</th>
                      <th className="px-6 py-4">Status</th>
                      <th className="px-6 py-4">Author</th>
                      <th className="px-6 py-4 text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {filteredPrs.length > 0 ? (
                      filteredPrs.map((pr) => (
                        <tr key={pr.id} className="group hover:bg-slate-50/80 transition-colors cursor-pointer" onClick={() => setSelectedPrId(pr.id)}>
                          <td className="px-6 py-4">
                            <div className="flex flex-col gap-1">
                              <div className="flex items-center gap-2">
                                <span className="text-xs font-mono font-medium text-slate-400">{pr.id}</span>
                                <span className="text-sm font-semibold group-hover:text-indigo-600 transition-colors">{pr.title}</span>
                              </div>
                              <div className="flex items-center gap-2 text-[11px] text-slate-500">
                                <span className="font-medium text-indigo-600/70">{pr.project_name}</span>
                                {pr.score && (
                                  <span className="flex items-center gap-1 text-emerald-600 ml-2 font-black bg-emerald-50 px-2 rounded shadow-sm">
                                    <Trophy size={10} /> {pr.score}/10
                                  </span>
                                )}
                              </div>
                            </div>
                          </td>
                          <td className="px-6 py-4">
                            <span className={`px-2.5 py-1 rounded-full text-[10px] font-bold border uppercase tracking-wider ${getStatusStyle(pr.status)}`}>
                              {pr.status}
                            </span>
                          </td>
                          <td className="px-6 py-4 text-xs font-medium">{pr.author_name}</td>
                          <td className="px-6 py-4 text-right"><MoreHorizontal size={18} className="text-slate-300 ml-auto" /></td>
                        </tr>
                      ))
                    ) : (
                      <tr>
                        <td colSpan="4" className="px-6 py-12 text-center text-slate-400 italic">No matching PRs found.</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </div>

        {/* Drawer */}
        {selectedPr && (
          <>
            <div className="absolute inset-0 bg-slate-900/20 backdrop-blur-[2px] z-10" onClick={() => setSelectedPrId(null)} />
            <div className="absolute right-0 top-0 h-full w-full max-w-xl bg-white shadow-2xl z-20 flex flex-col border-l border-slate-200 animate-in slide-in-from-right duration-300 text-slate-900">
              <div className="p-6 border-b border-slate-100 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className="bg-indigo-50 text-indigo-700 px-3 py-1 rounded text-xs font-mono font-bold">{selectedPr.id}</span>
                  <span className={`px-2.5 py-0.5 rounded-full text-[10px] font-bold border uppercase tracking-widest ${getStatusStyle(selectedPr.status)}`}>
                    {selectedPr.status}
                  </span>
                </div>
                <button onClick={() => setSelectedPrId(null)} className="p-2 hover:bg-slate-100 rounded-full transition-colors text-slate-400">
                  <X size={20} />
                </button>
              </div>

              <div className="flex-1 overflow-y-auto p-8 space-y-8 relative">
                {showScoring && (
                  <div className="absolute inset-0 z-30 bg-white flex flex-col items-center justify-center p-8 text-center animate-in fade-in slide-in-from-bottom-4">
                    <div className="bg-indigo-50 p-4 rounded-full mb-6 text-indigo-600">
                      <Trophy size={48} />
                    </div>
                    <h3 className="text-2xl font-bold mb-2">Review Completed!</h3>
                    <p className="text-slate-500 mb-8 max-w-sm px-4">Rate the quality of this Pull Request on a scale of 1 to 10.</p>
                    <div className="grid grid-cols-5 gap-3 mb-10">
                      {[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((num) => (
                        <button key={num} onClick={() => setSelectedScore(num)} className={`w-12 h-12 rounded-xl flex items-center justify-center text-lg font-black transition-all transform active:scale-90 border-2 ${(hoverScore || selectedScore) >= num ? 'bg-indigo-600 border-indigo-600 text-white' : 'bg-slate-50 border-slate-100 text-slate-400'}`}>{num}</button>
                      ))}
                    </div>
                    <div className="flex gap-3 w-full max-w-xs">
                      <button onClick={() => setShowScoring(false)} className="flex-1 py-3 text-sm font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-colors">Back</button>
                      <button disabled={selectedScore === 0} onClick={submitScore} className="flex-1 bg-indigo-600 text-white py-3 rounded-xl text-sm font-bold shadow-lg disabled:opacity-50">Approve & Score</button>
                    </div>
                  </div>
                )}

                <header className="space-y-4">
                  <h2 className="text-2xl font-bold leading-tight">{selectedPr.title}</h2>
                  <div className="flex items-center gap-4 text-sm">
                    <div className="flex items-center gap-2">
                      <img src={selectedPr.author_avatar || '/default-avatar.png'} className="w-6 h-6 rounded-full" />
                      <span className="font-medium">{selectedPr.author_name}</span>
                    </div>
                    <span className="text-slate-300">|</span>
                    <span className="text-slate-500">Recently Updated</span>
                  </div>
                </header>

                <section className="space-y-6">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Assign Project</label>
                      <select value={selectedPr.project_name || ''} onChange={(e) => updatePrProject(selectedPr.id, e.target.value)} className="w-full bg-white border-2 border-slate-200 rounded-xl px-3 py-2 text-xs outline-none cursor-pointer shadow-sm hover:shadow-md hover:border-indigo-300 focus:border-indigo-500 focus:ring-2 focus:ring-indigo-100 transition-all duration-200 appearance-none bg-no-repeat bg-right pr-8" style={{backgroundImage: "url(\"data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 16 16'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M4 6l4 4 4-4'/%3e%3c/svg%3e\")"}}>
                        {projects.map(p => <option key={p.id} value={p.name}>{p.name}</option>)}
                      </select>
                    </div>
                    <div className="space-y-2">
                      <label className="text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right block w-full">Current Score</label>
                      <div className="flex items-center justify-end gap-2 text-emerald-600 font-black h-[38px]">
                        {selectedPr.score ? <><Trophy size={14}/> {selectedPr.score}/10</> : <span className="text-slate-300 font-medium italic">No score</span>}
                      </div>
                    </div>
                  </div>

                  {selectedPr.status !== 'Approved' && (
                    <div className="space-y-3">
                      <label className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Review Progress</label>
                      <div className="flex gap-2">
                        {['Waiting', 'Reviewing', 'Action'].map((s) => (
                          <button key={s} onClick={() => updatePrStatus(selectedPr.id, s)} className={`flex-1 py-2 text-[11px] font-bold rounded-lg border transition-all ${selectedPr.status === s ? 'bg-white border-indigo-200 text-indigo-600 shadow-sm ring-2 ring-indigo-50' : 'bg-slate-50 border-slate-100 text-slate-400 hover:bg-slate-100'}`}>{s}</button>
                        ))}
                      </div>
                    </div>
                  )}
                </section>

                <section className="bg-slate-50 rounded-xl p-5 border border-slate-200 space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="text-xs font-semibold text-slate-400 uppercase tracking-widest">Branch Details</div>
                    <div className="flex items-center gap-2">
                      <button onClick={() => copyBranchDetails(selectedPr)} className="flex items-center gap-1 text-[10px] font-bold text-slate-500 hover:text-indigo-600 transition-colors">
                        <Copy size={11} />
                        Copy CR
                      </button>
                      <button onClick={handleSyncBranch} disabled={isSyncing} className="flex items-center gap-1 text-[10px] font-bold text-indigo-600 hover:text-indigo-700 disabled:opacity-50">
                        <RefreshCw size={12} className={isSyncing ? 'animate-spin' : ''} />
                        {isSyncing ? 'Syncing...' : 'Update from GitHub'}
                      </button>
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-slate-600 font-mono">
                    <span>into</span><span className="bg-white border border-slate-200 px-2 py-0.5 rounded text-slate-800 text-xs">main</span>
                    <span>from</span><span className="bg-indigo-50 border border-indigo-100 px-2 py-0.5 rounded text-indigo-700 text-xs">{selectedPr.branch}</span>
                  </div>
                </section>
              </div>

              <div className="p-6 border-t border-slate-100 bg-slate-50 flex gap-3">
                <button
                  onClick={() => handleCopyGitHubUrl(selectedPr)}
                  className="flex-1 bg-white border border-slate-200 py-3 rounded-xl text-sm font-bold hover:bg-slate-100 flex items-center justify-center gap-2 transition-colors"
                >
                  <Copy size={16} /> Copy GitHub URL
                </button>
                {selectedPr.status !== 'Approved' ? (
                  <>
                    <button
                      onClick={() => archivePr(selectedPr.id)}
                      className="bg-red-50 border border-red-200 text-red-600 py-3 px-4 rounded-xl text-sm font-bold hover:bg-red-100 flex items-center justify-center gap-2 transition-colors"
                    >
                      <Trash2 size={16} /> Archive
                    </button>
                    <button onClick={() => setShowScoring(true)} className="flex-1 bg-indigo-600 text-white py-3 rounded-xl text-sm font-bold hover:bg-indigo-700 shadow-lg flex items-center justify-center gap-2">
                      <Check size={18} /> Approve PR
                    </button>
                  </>
                ) : (
                  <button onClick={() => setShowScoring(true)} className="flex-1 bg-emerald-50 text-emerald-700 border border-emerald-100 py-3 rounded-xl text-sm font-black flex items-center justify-center gap-2">
                    <Check size={18} /> Rescore Approved PR
                  </button>
                )}
              </div>
            </div>
          </>
        )}

        {/* --- MODALS --- */}

        {/* Settings Modal */}
        {showSettings && (
          <Modal onClose={() => { setShowSettings(false); setTestStatus(null); }} title="Settings" icon={<Settings size={20} />}>
            <div className="space-y-6 text-slate-900">
              <div className="space-y-3">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest flex items-center gap-2">
                  <Key size={14} /> GitHub Personal Access Token
                </label>
                <div className="relative">
                  <input 
                    type={showKey ? "text" : "password"}
                    value={apiKey}
                    onChange={(e) => { setApiKey(e.target.value); setTestStatus(null); }}
                    className="w-full pl-4 pr-12 py-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-indigo-500/20 outline-none transition-all text-sm font-mono"
                    placeholder="ghp_xxxxxxxxxxxx or github_pat_xxxxxxxx"
                  />
                  <button onClick={() => setShowKey(!showKey)} className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600">
                    {showKey ? <EyeOff size={18} /> : <Eye size={18} />}
                  </button>
                </div>
                
                {/* 測試連結功能區塊 */}
                <div className="flex items-center justify-between gap-3 pt-1">
                  <button 
                    onClick={handleTestConnection}
                    disabled={isTesting || !apiKey}
                    className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all border ${
                      testStatus === 'success' 
                      ? 'bg-emerald-50 border-emerald-200 text-emerald-600' 
                      : testStatus === 'error'
                      ? 'bg-rose-50 border-rose-200 text-rose-600'
                      : 'bg-white border-slate-200 text-slate-600 hover:bg-slate-50 disabled:opacity-50'
                    }`}
                  >
                    {isTesting ? (
                      <RefreshCw size={14} className="animate-spin" />
                    ) : (
                      <Zap size={14} fill={testStatus === 'success' ? "currentColor" : "none"} />
                    )}
                    {isTesting ? 'Testing...' : testStatus === 'success' ? 'Connected' : 'Test Connection'}
                  </button>

                  {testStatus && (
                    <div className="flex items-center gap-1.5 animate-in fade-in slide-in-from-left-2">
                      {testStatus === 'success' ? (
                        <>
                          <Wifi size={14} className="text-emerald-500" />
                          <span className="text-[11px] font-bold text-emerald-600">API connection established</span>
                        </>
                      ) : (
                        <>
                          <WifiOff size={14} className="text-rose-500" />
                          <span className="text-[11px] font-bold text-rose-600">Invalid or expired token</span>
                        </>
                      )}
                    </div>
                  )}
                </div>
              </div>
              
              {/* Clear Database Section */}
              <div className="pt-4 border-t border-slate-100">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest flex items-center gap-2 mb-3">
                  <Trash2 size={14} /> Database Management
                </label>
                <p className="text-sm text-slate-600 mb-3">Clear all projects and PRs from the database. This cannot be undone.</p>
                <button
                  onClick={clearDatabase}
                  className="w-full py-2 px-4 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700 transition-colors flex items-center justify-center gap-2"
                >
                  <Trash2 size={16} /> Clear All Data
                </button>
              </div>

              <div className="pt-4 border-t border-slate-100 flex gap-3">
                <button onClick={() => setShowSettings(false)} className="flex-1 py-3 text-sm font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-colors">Close</button>
                <button onClick={handleSaveSettings} className="flex-1 bg-indigo-600 text-white py-3 rounded-xl text-sm font-bold hover:bg-indigo-700 shadow-lg flex items-center justify-center gap-2"><Save size={16} /> Save Changes</button>
              </div>
            </div>
          </Modal>
        )}

        {/* New PR Modal */}
        {showNewPr && (
          <Modal onClose={() => setShowNewPr(false)} title="Add Pull Request" icon={<Plus size={20} />}>
            <form onSubmit={submitNewPr} className="space-y-6 text-slate-900">
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest text-indigo-600">
                  <Github size={14} className="inline mr-1" />
                  GitHub PR URL
                </label>
                <input
                  required
                  autoFocus
                  type="url"
                  className="w-full p-4 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-indigo-500/20 outline-none text-sm shadow-sm font-mono"
                  placeholder="https://github.com/owner/repo/pull/123"
                  value={newPrData.url}
                  onChange={(e) => setNewPrData({...newPrData, url: e.target.value})}
                />
                <p className="text-xs text-slate-500 mt-1">
                  Copy and paste the GitHub PR URL from your browser
                </p>
              </div>
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest">Assign to Project</label>
                <select className="w-full p-4 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-indigo-500/20 outline-none text-sm cursor-pointer shadow-sm appearance-none" value={newPrData.project} onChange={(e) => setNewPrData({...newPrData, project: e.target.value})}>
                  {projects.map(p => <option key={p.id} value={p.name}>{p.name}</option>)}
                </select>
              </div>
              <div className="pt-4 flex gap-3">
                <button type="button" onClick={() => setShowNewPr(false)} className="flex-1 py-3.5 text-sm font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-colors">Cancel</button>
                <button type="submit" className="flex-1 bg-indigo-600 text-white py-3.5 rounded-xl text-sm font-bold shadow-lg">Add PR</button>
              </div>
            </form>
          </Modal>
        )}

        {/* Add Project Modal */}
        {showAddProject && (
          <Modal onClose={() => setShowAddProject(false)} title="Add New Project" icon={<FolderPlus size={20} />}>
            <form onSubmit={handleAddProject} className="space-y-4 text-slate-900">
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest">Project Name</label>
                <input required autoFocus className="w-full p-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-indigo-500/20 outline-none" placeholder="e.g. Design System" value={newProjectName} onChange={e => setNewProjectName(e.target.value)} />
              </div>
              <button type="submit" className="w-full py-3 bg-indigo-600 text-white rounded-xl font-bold shadow-lg hover:bg-indigo-700 transition-all text-white">Create Project</button>
            </form>
          </Modal>
        )}

        {/* Delete Project Confirmation Modal */}
        {showDeleteConfirm && projectToDelete && (
          <Modal onClose={() => { setShowDeleteConfirm(false); setProjectToDelete(null); }} title="Delete Project" icon={<Trash2 size={20} />}>
            <div className="space-y-4 text-slate-900">
              <div className="bg-red-50 border border-red-100 rounded-xl p-4">
                <div className="flex items-center gap-3 mb-2">
                  <div className="bg-red-100 p-2 rounded-full text-red-600">
                    <Trash2 size={16} />
                  </div>
                  <h4 className="font-bold text-red-800">Confirm Deletion</h4>
                </div>
                <p className="text-sm text-red-700">
                  Are you sure you want to delete the project <strong>"{projectToDelete.name}"</strong>?
                </p>
                <p className="text-xs text-red-600 mt-2 italic">
                  This action cannot be undone.
                </p>
              </div>
              <div className="flex gap-3">
                <button
                  onClick={() => { setShowDeleteConfirm(false); setProjectToDelete(null); }}
                  className="flex-1 py-3 text-sm font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmDeleteProject}
                  className="flex-1 bg-red-600 text-white py-3 rounded-xl text-sm font-bold hover:bg-red-700 shadow-lg transition-all"
                >
                  Delete Project
                </button>
              </div>
            </div>
          </Modal>
        )}

        {/* Error Modal */}
        {errorModal.show && (
          <Modal onClose={() => setErrorModal({show: false, title: '', message: ''})} title={errorModal.title} icon={<AlertCircle size={20} />}>
            <div className="space-y-4 text-slate-900">
              <div className="bg-red-50 border border-red-100 rounded-xl p-4">
                <div className="flex items-start gap-3">
                  <div className="bg-red-100 p-2 rounded-full text-red-600 shrink-0">
                    <AlertCircle size={16} />
                  </div>
                  <div className="space-y-2 flex-1">
                    <h4 className="font-bold text-red-800">{errorModal.title}</h4>
                    <div className="text-sm text-red-700 whitespace-pre-wrap break-words">
                      {errorModal.message}
                    </div>
                  </div>
                </div>
              </div>
              <div className="flex justify-end">
                <button
                  onClick={() => setErrorModal({show: false, title: '', message: ''})}
                  className="px-6 py-3 bg-red-600 text-white rounded-xl text-sm font-bold hover:bg-red-700 shadow-lg transition-all"
                >
                  OK
                </button>
              </div>
            </div>
          </Modal>
        )}
      </main>
    </div>
  );
};

// UI Components
const NavItem = ({ icon, label, active, badge, onClick, small }) => (
  <button onClick={onClick} className={`w-full flex items-center justify-between px-3 py-2 rounded-lg transition-all group/item ${active ? 'bg-indigo-50 text-indigo-700 shadow-sm shadow-indigo-50 ring-1 ring-indigo-200' : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900'} ${small ? 'py-1.5' : ''}`}>
    <div className="flex items-center gap-3">
      {icon && <span className={active ? 'text-indigo-600' : 'text-slate-400 group-hover/item:text-slate-600'}>{icon}</span>}
      <span className={`${small ? 'text-[13px]' : 'text-[14px] font-medium'} truncate max-w-[140px]`}>{label}</span>
    </div>
    {badge && badge !== "0" && <span className="bg-orange-100 text-orange-700 text-[10px] font-bold px-1.5 py-0.5 rounded-full">{badge}</span>}
  </button>
);

const StatCard = ({ label, value, icon, color, isActive, onClick }) => (
  <button 
    onClick={onClick}
    className={`bg-white p-5 rounded-xl border-2 transition-all hover:-translate-y-1 flex items-start justify-between w-full text-left ${
      isActive ? 'border-indigo-600 ring-4 ring-indigo-50 shadow-md' : 'border-transparent shadow-sm hover:shadow-md'
    }`}
  >
    <div>
      <p className="text-slate-400 text-[10px] font-bold uppercase tracking-widest mb-1">{label}</p>
      <p className="text-2xl font-black tracking-tight">{value}</p>
    </div>
    <div className={`p-2.5 rounded-xl ${color}`}>
      {icon}
    </div>
  </button>
);

const Modal = ({ onClose, title, icon, children }) => (
  <div className="fixed inset-0 flex items-center justify-center z-[100] px-4 text-slate-900">
    <div className="absolute inset-0 bg-slate-900/40 backdrop-blur-sm animate-in fade-in" onClick={onClose} />
    <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200">
      <div className="p-6 border-b border-slate-100 flex items-center justify-between">
        <div className="flex items-center gap-3">
          {icon && <div className="bg-indigo-50 p-2 rounded-lg text-indigo-600">{icon}</div>}
          <h3 className="font-bold text-slate-800">{title}</h3>
        </div>
        <button onClick={onClose} className="p-1 hover:bg-slate-100 rounded-full transition-colors text-slate-400 hover:text-slate-600"><X size={20} /></button>
      </div>
      <div className="p-6">
        {children}
      </div>
    </div>
  </div>
);

export default App;
